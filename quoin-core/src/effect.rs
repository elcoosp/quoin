//! Core-native reactive effects: re-run when the signals they read change.
//!
//! quoin's derived layer ([`derived`](crate::derived), [`memo`](crate::memo))
//! is *pull-based*: a computed value recomputes when read. Effects complete
//! the reactive triangle with the *push* side — "when any signal I read
//! changes, run this closure" — as a portable [`std`]-only utility, following
//! the same pattern as [`CancellationToken`](crate::CancellationToken):
//! a core primitive no adapter has to implement.
//!
//! # Dependency tracking
//!
//! During an effect run every tracked-cell read (see [`TrackedSignal`]) is
//! recorded. The effect subscribes to exactly the cells it read *on that
//! run*; the dependency set is rebuilt on every re-run, so conditional reads
//! produce dynamic dependencies. Reads outside any effect run pay a single
//! thread-local flag check — the same zero-cost-gate pattern the devtools
//! wrapper uses.
//!
//! # Batch integration
//!
//! Mutations inside [`batch::with`](crate::batch::with) enqueue effect
//! re-runs instead of running them inline; the batch end flushes the queue,
//! so N writes produce at most one re-run per effect. This mirrors the
//! existing "N mutations, one update request" batch contract.
//!
//! # Lifecycle
//!
//! [`create_effect`] runs the closure once immediately, then re-runs it on
//! every mutation of a tracked dependency. The returned [`EffectHandle`]
//! *detaches* on drop: unlike a [`JoinHandle`](crate::JoinHandle), dropping
//! it does **not** dispose the effect (an effect has no result to orphan,
//! and fire-and-forget observation is the common case). Bound effects to a
//! component with [`EffectScope`], whose `Drop` disposes every effect it
//! created, or dispose explicitly with [`EffectHandle::dispose`].
//!
//! An effect that writes one of its own dependencies does **not** recurse or
//! deadlock: the self-write is deferred, and the effect re-runs once the
//! current run has finished (a `set_if_eq` write that does not change the
//! value does not notify and breaks such cycles entirely). Panics inside an
//! effect body propagate to the caller that triggered the run; the runtime
//! itself stays consistent — the panicking run's tracking frame is popped on
//! the way out (the thread keeps tracking later effects correctly), the
//! effect keeps its previous dependency set until a successful run rebuilds
//! it, and a creation whose *initial* run panics is fully rolled back (no
//! live effect, no registry entry, no disabled zero-cost gate).
//!
//! # Example
//!
//! ```
//! use quoin_core::prelude::*;
//!
//! let count = quoin_core::test_support::static_signal(1u32);
//! let seen = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
//!
//! let seen2 = seen.clone();
//! let count2 = count.clone();
//! let handle = quoin_core::create_effect(move || {
//!     let value = count2.get();
//!     seen2.store(value as usize, std::sync::atomic::Ordering::SeqCst);
//! });
//!
//! // The effect ran once immediately.
//! assert_eq!(seen.load(std::sync::atomic::Ordering::SeqCst), 1);
//!
//! count.set(7);
//! assert_eq!(seen.load(std::sync::atomic::Ordering::SeqCst), 7);
//!
//! handle.dispose();
//! count.set(9); // no longer observed
//! assert_eq!(seen.load(std::sync::atomic::Ordering::SeqCst), 7);
//! ```

use crate::signal::Signal;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

/// Effect node state: subscribed and re-runnable.
const ACTIVE: u8 = 0;
/// Effect node state: disposed; never scheduled or re-run again.
const DISPOSED: u8 = 1;

thread_local! {
    /// Per-thread tracking gate: `true` while an effect body is running on
    /// this thread. Deliberately thread-local — a global flag would let one
    /// thread's frame teardown untrack another thread's in-flight effect
    /// body (a real race the effect conformance suite caught: missed
    /// subscriptions → missed re-runs under parallel tests). Const init
    /// keeps the idle-path cost at one register-relative TLS load.
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    /// Stack of active tracking frames; the top frame collects the reads of
    /// the effect that is currently running (nested when an effect body
    /// creates another effect).
    static FRAMES: RefCell<Vec<Vec<DepHandle>>> = const { RefCell::new(Vec::new()) };
    /// Effects scheduled for a re-run, in schedule order (FIFO).
    static PENDING: RefCell<VecDeque<Arc<EffectNode>>> = const { RefCell::new(VecDeque::new()) };
    /// Flush recursion depth; only the outermost flush drains the queue.
    static FLUSH_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Number of effect nodes that are created and not yet disposed. Mutations
/// skip the per-dependency subscriber lock entirely while this is zero,
/// keeping unobserved `set`/`get` at their historical cost.
static LIVE_EFFECTS: AtomicUsize = AtomicUsize::new(0);
/// Registry of detached effects. Effects are detached-by-default (dropping
/// the [`EffectHandle`] neither pauses nor disposes them), so the runtime
/// owns every node here until [`EffectHandle::dispose`] removes it. Scope
/// effects keep their entry too — disposal always goes through `dispose`.
///
/// Removal is O(1): every node stores its position (`EffectNode::slot`) and
/// `swap_remove` fixes up the moved node's slot. All slot reads and writes
/// happen under this lock, so the bookkeeping is race-free by construction.
static DETACHED: Mutex<Vec<Arc<EffectNode>>> = Mutex::new(Vec::new());

/// Sentinel slot for a node that is not registered (creation failed, or the
/// registry lock was poisoned at registration time).
const UNREGISTERED: usize = usize::MAX;

// ---------------------------------------------------------------------------
// Dependency cell
// ---------------------------------------------------------------------------

/// The subscriber list behind one tracked signal cell.
struct DepCell {
    subscribers: Mutex<Vec<Weak<EffectNode>>>,
}

/// An opaque handle to a signal's dependency cell.
///
/// Cloning shares the same cell. This is the extension-point primitive for
/// adapter and cell authors: a custom signal type that wants core effect
/// support stores one [`DepHandle`] per cell, calls [`track_read`](DepHandle::track_read)
/// on every read, [`notify`](DepHandle::notify) after every published
/// mutation, and implements [`TrackedSignal`] by returning a clone. Reads
/// outside an effect run pay one thread-local flag check.
#[derive(Clone)]
pub struct DepHandle {
    cell: Arc<DepCell>,
}

impl DepHandle {
    /// Creates an empty dependency cell with no subscribers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cell: Arc::new(DepCell {
                subscribers: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Identity comparison: `true` when both handles point at the same cell.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.cell, &other.cell)
    }

    /// Records a read of the owning cell in the innermost tracking frame.
    ///
    /// One thread-local flag check when no effect is running.
    #[inline]
    pub fn track_read(&self) {
        if !TRACKING.with(Cell::get) {
            return;
        }
        FRAMES.with(|frames| {
            if let Some(top) = frames.borrow_mut().last_mut() {
                top.push(self.clone());
            }
        });
    }

    /// Schedules every subscribed effect for a re-run.
    ///
    /// The subscriber lock is released **before** effects are scheduled:
    /// a scheduled effect's re-run resubscribes (locks this same cell) when
    /// it rebuilds its dependency set, so holding the lock across `schedule`
    /// would self-deadlock on the thread that mutated the signal.
    pub fn notify(&self) {
        if LIVE_EFFECTS.load(Ordering::Relaxed) == 0 {
            return;
        }
        let subscribers = {
            let mut guard = self.lock_subscribers();
            if guard.is_empty() {
                return;
            }
            let nodes: Vec<Arc<EffectNode>> = guard.iter().filter_map(Weak::upgrade).collect();
            guard.retain(|weak| weak.strong_count() > 0);
            nodes
        };
        for node in subscribers {
            node.schedule();
        }
    }

    fn lock_subscribers(&self) -> MutexGuard<'_, Vec<Weak<EffectNode>>> {
        self.cell
            .subscribers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn add_subscriber(&self, node: &Arc<EffectNode>) {
        self.lock_subscribers().push(Arc::downgrade(node));
    }

    /// Removes `node` from the subscriber list (and prunes dead entries).
    fn remove_subscriber(&self, node: &Arc<EffectNode>) {
        self.lock_subscribers().retain(|weak| {
            weak.upgrade()
                .is_some_and(|candidate| !Arc::ptr_eq(&candidate, node))
        });
    }
}

impl Default for DepHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DepHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DepHandle").finish_non_exhaustive()
    }
}

/// A signal whose mutations the core effect runtime can observe.
///
/// Implemented by the core signal cells ([`StaticSignal`](crate::test_support::StaticSignal),
/// [`NotifierSignal`](crate::NotifierSignal), the testkit `TestSignal`, and
/// [`AnySignal`](crate::AnySignal) — which forwards the inner cell when built
/// with `new_tracked` and panics for plain erasures). Reads of these cells
/// inside an effect body register a dependency; a mutation schedules the
/// effect for a re-run.
///
/// Framework-native signal types on the fine-grained adapters (Leptos,
/// Dioxus, Floem) have their own reactive graphs and do not implement this
/// trait; effects there track quoin-created cells only. The pull-based
/// adapters (GPUI, Xilem) alias [`NotifierSignal`], so core effects observe
/// their signals natively.
pub trait TrackedSignal<T: Clone + 'static>: Signal<T> {
    /// The dependency cell notified when this signal's value is written.
    fn dep_handle(&self) -> DepHandle;
}

// ---------------------------------------------------------------------------
// Effect node and handle
// ---------------------------------------------------------------------------

struct EffectNode {
    state: AtomicU8,
    queued: AtomicBool,
    deps: Mutex<Vec<DepHandle>>,
    run: Mutex<Box<dyn FnMut() + Send>>,
    /// Position in the detached registry (valid while registered); all
    /// accesses happen under the registry lock.
    slot: AtomicUsize,
}

impl EffectNode {
    /// Runs the body once, rebuilding the dependency subscription set.
    ///
    /// Unwind safety: the tracking frame is an RAII guard, so a panicking
    /// body pops its own frame on the way out — the thread's tracking state
    /// (gate + frame stack) is restored before the panic escapes. The
    /// subscription rebuild is skipped on the unwind path: the effect keeps
    /// its previous dependency set until a successful run rebuilds it, and
    /// [`EffectHandle::dispose`] remains the escape hatch.
    fn run_once(self: &Arc<Self>) {
        if self.state.load(Ordering::Acquire) == DISPOSED {
            return;
        }
        let mut run = self.run.lock().unwrap_or_else(PoisonError::into_inner);
        if self.state.load(Ordering::Acquire) == DISPOSED {
            return;
        }
        let frame = FrameGuard::enter();
        (run)();
        let reads = frame.finish();
        drop(run);

        let mut deps = self.lock_deps();
        for old in deps.drain(..) {
            old.remove_subscriber(self);
        }
        for dep in &reads {
            dep.add_subscriber(self);
        }
        *deps = reads;
    }

    /// Marks the node for a re-run and flushes it if no flush is in progress.
    fn schedule(self: Arc<Self>) {
        if self.state.load(Ordering::Acquire) == DISPOSED {
            return;
        }
        if self.queued.swap(true, Ordering::AcqRel) {
            return;
        }
        PENDING.with(|pending| pending.borrow_mut().push_back(self));
        if !crate::batch::in_batch() && !is_flushing() {
            flush();
        }
    }

    fn lock_deps(&self) -> MutexGuard<'_, Vec<DepHandle>> {
        self.deps.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for EffectNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectNode").finish_non_exhaustive()
    }
}

/// Owning handle to a running effect.
///
/// Clone shares the effect. Dropping the handle **detaches** the effect (it
/// keeps running); call [`dispose`](EffectHandle::dispose) to stop it, or
/// create it through an [`EffectScope`] for group disposal.
#[derive(Clone)]
pub struct EffectHandle {
    node: Arc<EffectNode>,
}

impl EffectHandle {
    /// Disposes the effect: unsubscribes it from every dependency, removes
    /// it from the detached registry, and prevents any further run.
    /// Idempotent.
    pub fn dispose(&self) {
        if self.node.state.swap(DISPOSED, Ordering::AcqRel) == DISPOSED {
            return;
        }
        let deps = std::mem::take(&mut *self.node.lock_deps());
        for dep in deps {
            dep.remove_subscriber(&self.node);
        }
        self.node.queued.store(false, Ordering::Release);
        LIVE_EFFECTS.fetch_sub(1, Ordering::Relaxed);
        remove_from_registry(&self.node);
    }

    /// `true` after [`dispose`](EffectHandle::dispose) was called.
    #[must_use]
    pub fn is_disposed(&self) -> bool {
        self.node.state.load(Ordering::Acquire) == DISPOSED
    }

    /// `true` while the effect is subscribed and able to re-run.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_disposed()
    }
}

impl fmt::Debug for EffectHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectHandle")
            .field("disposed", &self.is_disposed())
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Tracking frames
// ---------------------------------------------------------------------------

fn push_frame() {
    FRAMES.with(|frames| {
        frames.borrow_mut().push(Vec::new());
        TRACKING.with(|gate| gate.set(true));
    });
}

fn pop_frame() -> Vec<DepHandle> {
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        let frame = frames.pop().expect("pop_frame without push_frame");
        if frames.is_empty() {
            TRACKING.with(|gate| gate.set(false));
        }
        frame
    })
}

/// RAII twin of `push_frame`/`pop_frame`: an effect run opens a tracking
/// frame and must close it on *every* exit path. A panicking body unwinds
/// through [`FrameGuard::drop`], which pops the partial frame and restores
/// the tracking gate — without the guard, the thread would keep `TRACKING`
/// stuck on `true` and an ever-growing stale frame forever (every later
/// tracked read on that thread would pay full frame bookkeeping and leak a
/// `DepHandle` clone into the dead frame).
struct FrameGuard {
    active: bool,
}

impl FrameGuard {
    fn enter() -> Self {
        push_frame();
        Self { active: true }
    }

    /// Successful-run path: pops the frame and returns the recorded reads.
    fn finish(mut self) -> Vec<DepHandle> {
        self.active = false;
        pop_frame()
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        if self.active {
            // Unwind path: discard the partial frame; the reads are not
            // subscribed (the subscription rebuild is skipped after a panic).
            drop(pop_frame());
        }
    }
}

// ---------------------------------------------------------------------------
// Registry (slot-indexed, O(1) removal)
// ---------------------------------------------------------------------------

fn register_in_registry(node: &Arc<EffectNode>) {
    if let Ok(mut registry) = DETACHED.lock() {
        let slot = registry.len();
        registry.push(Arc::clone(node));
        node.slot.store(slot, Ordering::Relaxed);
    }
    // A poisoned registry lock skips registration (the effect still runs);
    // the node keeps the UNREGISTERED sentinel, so a later `dispose` will
    // not touch the registry.
}

fn remove_from_registry(node: &Arc<EffectNode>) {
    if let Ok(mut registry) = DETACHED.lock() {
        let slot = node.slot.swap(UNREGISTERED, Ordering::Relaxed);
        if slot >= registry.len() || !Arc::ptr_eq(&registry[slot], node) {
            return; // never registered (or already removed)
        }
        let last = registry.len() - 1;
        registry.swap_remove(slot);
        if slot < last {
            // The last node moved into `slot`: fix its position up.
            registry[slot].slot.store(slot, Ordering::Relaxed);
        }
    }
}

/// Unwinds the [`create_effect`] registration when the initial run panics:
/// marks the node disposed, releases the zero-cost gate, and removes the
/// registry entry — a failed creation must leave no trace.
struct RegistrationGuard {
    node: Arc<EffectNode>,
    armed: bool,
}

impl RegistrationGuard {
    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        self.node.state.store(DISPOSED, Ordering::Release);
        let deps = std::mem::take(&mut *self.node.lock_deps());
        for dep in deps {
            dep.remove_subscriber(&self.node);
        }
        LIVE_EFFECTS.fetch_sub(1, Ordering::Relaxed);
        remove_from_registry(&self.node);
    }
}

// ---------------------------------------------------------------------------
// Flush queue
// ---------------------------------------------------------------------------

struct FlushGuard;

impl FlushGuard {
    fn enter() -> Self {
        FLUSH_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        FLUSH_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

fn is_flushing() -> bool {
    FLUSH_DEPTH.with(|depth| depth.get() > 0)
}

/// Drains the pending re-run queue (FIFO) until it is empty.
///
/// Nested calls (an effect body scheduling while a flush is already draining
/// on this thread) return immediately; the outer loop picks the work up.
pub(crate) fn flush() {
    if is_flushing() {
        return;
    }
    let _guard = FlushGuard::enter();
    loop {
        let next = PENDING.with(|pending| pending.borrow_mut().pop_front());
        let Some(node) = next else { break };
        if node.queued.swap(false, Ordering::AcqRel) {
            node.run_once();
        }
    }
}

// ---------------------------------------------------------------------------
// Public constructors
// ---------------------------------------------------------------------------

/// Creates an effect: runs `run` once immediately, then re-runs it whenever
/// any signal cell it read on the previous run is mutated.
///
/// See the [module docs](self) for the tracking, batching, and lifecycle
/// contract.
///
/// # Panics
///
/// Panics if the initial run panics (the panic propagates to the caller).
/// The failed creation is rolled back — no effect exists afterwards.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let width = quoin_core::test_support::static_signal(10u32);
/// let height = quoin_core::test_support::static_signal(2u32);
/// let area = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
///
/// let area2 = area.clone();
/// let (w, h) = (width.clone(), height.clone());
/// let _handle = quoin_core::create_effect(move || {
///     let value = w.get() * h.get();
///     area2.store(value as usize, std::sync::atomic::Ordering::SeqCst);
/// });
/// assert_eq!(area.load(std::sync::atomic::Ordering::SeqCst), 20);
/// height.set(3);
/// assert_eq!(area.load(std::sync::atomic::Ordering::SeqCst), 30);
/// width.set(5); // both dependencies were read: both are tracked
/// assert_eq!(area.load(std::sync::atomic::Ordering::SeqCst), 15);
/// ```
pub fn create_effect<F>(run: F) -> EffectHandle
where
    F: FnMut() + Send + 'static,
{
    let node = Arc::new(EffectNode {
        state: AtomicU8::new(ACTIVE),
        queued: AtomicBool::new(false),
        deps: Mutex::new(Vec::new()),
        run: Mutex::new(Box::new(run)),
        slot: AtomicUsize::new(UNREGISTERED),
    });
    LIVE_EFFECTS.fetch_add(1, Ordering::Relaxed);
    register_in_registry(&node);
    {
        // Defer inline flushes while the initial run executes: a body that
        // writes a dependency must not deadlock on its own run mutex. The
        // registration guard undoes the registration if the initial run
        // panics, so a failed creation leaves no live effect, no registry
        // entry, and no disabled zero-cost gate behind.
        let registration = RegistrationGuard {
            node: Arc::clone(&node),
            armed: true,
        };
        let _guard = FlushGuard::enter();
        node.run_once();
        registration.disarm();
    }
    flush();
    EffectHandle { node }
}

/// Runs `on_change` whenever `signal` is written (but not for the initial
/// value), by subscribing a [`create_effect`] to the signal's dependency
/// cell.
///
/// This is the explicit-dependency counterpart of reading inside
/// [`create_effect`]; it fires on every *write* notification, including
/// writes that store an equal value — use
/// [`set_eq`](crate::SignalExt::set_eq) on the writer side to suppress
/// no-op writes.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let name = quoin_core::test_support::static_signal("ada".to_string());
/// let last = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
///
/// let last2 = last.clone();
/// let name2 = name.clone();
/// let _handle = quoin_core::watch(&name2, move |value: &String| {
///     *last2.lock().unwrap() = value.clone();
/// });
///
/// assert!(last.lock().unwrap().is_empty(), "not fired for the initial value");
/// name.set("grace".to_string());
/// assert_eq!(last.lock().unwrap().as_str(), "grace");
/// ```
pub fn watch<S, T, F>(signal: &S, mut on_change: F) -> EffectHandle
where
    S: TrackedSignal<T> + Send + 'static,
    T: Clone + 'static,
    F: FnMut(&T) + Send + 'static,
{
    let mut armed = false;
    let owned = signal.clone();
    create_effect(move || {
        owned.with(|value| {
            if armed {
                on_change(value);
            }
        });
        armed = true;
    })
}

// ---------------------------------------------------------------------------
// Effect scope
// ---------------------------------------------------------------------------

/// A group of effects disposed together.
///
/// The component-level owner: `Drop` disposes every effect created through
/// the scope, so binding effects to a UI component's lifetime is a field
/// declaration, not manual cleanup. Scope methods take `&self`, so the
/// scope can be shared via `Rc`/`Arc` or live behind `&mut self`.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let count = quoin_core::test_support::static_signal(0u32);
/// let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
///
/// let scope = quoin_core::EffectScope::new();
/// let (count2, runs2) = (count.clone(), runs.clone());
/// scope.create_effect(move || {
///     count2.get();
///     runs2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
/// });
/// assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 1);
///
/// drop(scope); // disposes the effect
/// count.set(1);
/// assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 1);
/// ```
pub struct EffectScope {
    handles: Mutex<Vec<EffectHandle>>,
}

impl Default for EffectScope {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectScope {
    /// Creates an empty scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(Vec::new()),
        }
    }

    /// Creates an effect owned by this scope; see [`create_effect`].
    pub fn create_effect<F>(&self, run: F) -> EffectHandle
    where
        F: FnMut() + Send + 'static,
    {
        let handle = create_effect(run);
        self.register(handle.clone());
        handle
    }

    /// Creates a change watcher owned by this scope; see [`watch`].
    pub fn watch<S, T, F>(&self, signal: &S, on_change: F) -> EffectHandle
    where
        S: TrackedSignal<T> + Send + 'static,
        T: Clone + 'static,
        F: FnMut(&T) + Send + 'static,
    {
        let handle = watch(signal, on_change);
        self.register(handle.clone());
        handle
    }

    fn register(&self, handle: EffectHandle) {
        self.lock_handles().push(handle);
    }

    /// Disposes every effect in the scope (idempotent per effect). The scope
    /// stays usable: new effects can still be created through it.
    pub fn dispose_all(&self) {
        for handle in self.lock_handles().iter() {
            handle.dispose();
        }
    }

    /// Number of effects created through this scope, including disposed
    /// ones (the count is historical).
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock_handles().len()
    }

    /// `true` when no effect was created through this scope.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock_handles(&self) -> MutexGuard<'_, Vec<EffectHandle>> {
        self.handles.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for EffectScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectScope")
            .field("effects", &self.len())
            .finish_non_exhaustive()
    }
}

impl Drop for EffectScope {
    fn drop(&mut self) {
        self.dispose_all();
    }
}

// ---------------------------------------------------------------------------
// Batch hook
// ---------------------------------------------------------------------------

/// Called by [`crate::batch::BatchGuard`] when the outermost batch ends:
/// flush the deferred re-runs now.
pub(crate) fn on_batch_end() {
    flush();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::static_signal;
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    fn counter() -> Arc<AtomicUsize> {
        Arc::new(AtomicUsize::new(0))
    }

    fn runs(c: &Arc<AtomicUsize>) -> usize {
        c.load(Ordering::SeqCst)
    }

    /// Number of nodes currently registered in the detached registry.
    fn registry_len() -> usize {
        DETACHED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }

    /// The registry slot invariant: every registered node's stored slot
    /// equals its index (kept under the registry lock by create/dispose).
    fn registry_invariant_holds() -> bool {
        let registry = DETACHED.lock().unwrap_or_else(PoisonError::into_inner);
        registry
            .iter()
            .enumerate()
            .all(|(index, node)| node.slot.load(Ordering::Relaxed) == index)
    }

    /// Serializes tests that observe the process-global effect counters
    /// (`LIVE_EFFECTS`, `DETACHED`). The project runner (cargo-nextest)
    /// executes every test in its own process; this guard keeps plain
    /// `cargo test` in-process runs deterministic as well (same pattern as
    /// devtools' `global_flag_lock`).
    fn global_state_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    #[test]
    fn effect_runs_immediately_on_create() {
        let _serial = global_state_lock();
        let count = counter();
        let c2 = count.clone();
        let _handle = create_effect(move || {
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 1, "initial run happens at creation");
    }

    #[test]
    fn effect_reruns_on_dependency_mutation() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let _handle = create_effect(move || {
            s2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 1);
        source.set(2);
        assert_eq!(runs(&count), 2, "mutation of a read cell re-runs");
        source.set(3);
        assert_eq!(runs(&count), 3);
    }

    #[test]
    fn effect_reads_fresh_value_each_run() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (s2, seen2) = (source.clone(), seen.clone());
        let _handle = create_effect(move || {
            seen2.lock().unwrap().push(s2.get());
        });
        source.set(2);
        source.set(3);
        assert_eq!(*seen.lock().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn effect_tracks_multiple_dependencies() {
        let _serial = global_state_lock();
        let a = static_signal(0u32);
        let b = static_signal(0u32);
        let count = counter();
        let (a2, b2, c2) = (a.clone(), b.clone(), count.clone());
        let _handle = create_effect(move || {
            a2.get();
            b2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 1);
        a.set(1);
        assert_eq!(runs(&count), 2);
        b.set(1);
        assert_eq!(runs(&count), 3);
    }

    #[test]
    fn effect_reads_are_visible_after_mutation() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let seen = counter();
        let (s2, seen2) = (source.clone(), seen.clone());
        let _handle = create_effect(move || {
            seen2.store(s2.get() as usize, Ordering::SeqCst);
        });
        source.set(42);
        assert_eq!(seen.load(Ordering::SeqCst), 42, "re-run observes new value");
    }

    #[test]
    fn effect_dependency_set_is_dynamic() {
        let _serial = global_state_lock();
        let mode = static_signal(true);
        let on = static_signal("on".to_string());
        let off = static_signal("off".to_string());
        let seen = Arc::new(Mutex::new(String::new()));
        let count = counter();

        let (m2, o2, f2, seen2, c2) = (
            mode.clone(),
            on.clone(),
            off.clone(),
            seen.clone(),
            count.clone(),
        );
        let _handle = create_effect(move || {
            let text = if m2.get() { o2.get() } else { f2.get() };
            *seen2.lock().unwrap() = text;
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 1);

        // Tracked: both branch inputs were read on the first run? No — only
        // `mode` and `on`. Mutating `off` must not re-run.
        off.set("OFF".to_string());
        assert_eq!(runs(&count), 1, "untracked branch input does not notify");

        on.set("ON".to_string());
        assert_eq!(runs(&count), 2);

        // Flip the branch: the dependency set now contains `off`, not `on`.
        mode.set(false);
        assert_eq!(runs(&count), 3);
        assert_eq!(*seen.lock().unwrap(), "OFF");
        on.set("on2".to_string());
        assert_eq!(runs(&count), 3, "old branch input unsubscribed");
        off.set("off2".to_string());
        assert_eq!(runs(&count), 4, "new branch input tracked");
    }

    #[test]
    fn derived_over_tracked_source_triggers_effect() {
        let _serial = global_state_lock();
        let source = static_signal(2u32);
        let doubled = crate::derived(&source, |v| v * 2);
        let seen = counter();
        let (d2, seen2) = (doubled.clone(), seen.clone());
        let _handle = create_effect(move || {
            seen2.store(d2.get() as usize, Ordering::SeqCst);
        });
        assert_eq!(seen.load(Ordering::SeqCst), 4);
        source.set(5);
        assert_eq!(
            seen.load(Ordering::SeqCst),
            10,
            "tracked through the derive"
        );
    }

    #[test]
    fn effect_dispose_stops_reruns() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let handle = create_effect(move || {
            s2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert!(handle.is_active());
        handle.dispose();
        assert!(handle.is_disposed());
        source.set(2);
        assert_eq!(runs(&count), 1, "disposed effect never re-runs");
        handle.dispose(); // idempotent
        assert!(handle.is_disposed());
    }

    #[test]
    fn cloned_handle_shares_dispose_state() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let handle = create_effect(move || {
            s2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        let clone = handle.clone();
        clone.dispose();
        assert!(handle.is_disposed(), "clone disposes the same effect");
    }

    #[test]
    fn dropping_handle_detaches_but_keeps_effect_running() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        {
            let _handle = create_effect(move || {
                s2.get();
                c2.fetch_add(1, Ordering::SeqCst);
            });
        }
        source.set(2);
        assert_eq!(runs(&count), 2, "drop(handle) must not dispose the effect");
    }

    #[test]
    fn effect_scope_disposes_children_on_drop() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let scope = EffectScope::new();
        scope.create_effect(move || {
            s2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(scope.len(), 1);
        assert!(!scope.is_empty());
        assert_eq!(runs(&count), 1);
        drop(scope);
        source.set(2);
        assert_eq!(runs(&count), 1, "scope drop disposed the effect");
    }

    #[test]
    fn effect_scope_dispose_all_is_explicit_and_reusable() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let scope = EffectScope::new();
        scope.create_effect(move || {
            s2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        scope.dispose_all();
        source.set(2);
        assert_eq!(runs(&count), 1);
        // The scope is still usable after dispose_all.
        scope.create_effect(|| {});
        assert_eq!(scope.len(), 2);
    }

    #[test]
    fn batch_coalesces_effect_reruns() {
        let _serial = global_state_lock();
        let a = static_signal(0u32);
        let b = static_signal(0u32);
        let count = counter();
        let (a2, b2, c2) = (a.clone(), b.clone(), count.clone());
        let _handle = create_effect(move || {
            let _ = (a2.get(), b2.get());
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 1);
        crate::batch::with(&crate::test_support::StaticContext, || {
            a.set(1);
            b.set(1);
            a.set(2);
            assert_eq!(runs(&count), 1, "no re-run inside the batch");
        });
        assert_eq!(runs(&count), 2, "N batched writes → one re-run");
    }

    #[test]
    fn self_write_defers_not_deadlocks() {
        let _serial = global_state_lock();
        // The initial run cannot self-trigger (its subscriptions install
        // after the body), so an external mutation starts the chain. From
        // the second run on, a self-write is deferred: the effect re-runs
        // after the current run finishes instead of recursing.
        let trigger = static_signal(0u32);
        let state = static_signal(0u32);
        let count = counter();
        let (t2, s2, c2) = (trigger.clone(), state.clone(), count.clone());
        let _handle = create_effect(move || {
            t2.get();
            c2.fetch_add(1, Ordering::SeqCst);
            let n = s2.get();
            if n < 2 {
                s2.set(n + 1); // writes its own dependency
            }
        });
        assert_eq!(runs(&count), 1, "initial run: no self-trigger yet");
        assert_eq!(state.get(), 1);

        trigger.set(1); // re-run: self-write defers, then the re-run stops at n=2
        assert_eq!(
            runs(&count),
            3,
            "external re-run + one deferred self-write re-run (bounded by the body)"
        );
        assert_eq!(state.get(), 2);
    }

    #[test]
    fn chained_effects_propagate_in_fifo_order() {
        let _serial = global_state_lock();
        let a = static_signal(0u32);
        let b = static_signal(0u32);
        let log = Arc::new(Mutex::new(Vec::new()));

        let (a2, b2, log2) = (a.clone(), b.clone(), log.clone());
        let first = create_effect(move || {
            let value = a2.get();
            b2.set(value * 10); // writes b without reading it: no self-dependency
            log2.lock().unwrap().push("e1");
        });
        let (b3, log3) = (b.clone(), log.clone());
        let _second = create_effect(move || {
            b3.get();
            log3.lock().unwrap().push("e2");
        });
        assert_eq!(
            *log.lock().unwrap(),
            vec!["e1", "e2"],
            "initial runs in creation order"
        );

        log.lock().unwrap().clear();
        a.set(1);
        assert_eq!(
            *log.lock().unwrap(),
            vec!["e1", "e2"],
            "downstream effect runs after the upstream one, same flush"
        );
        assert_eq!(b.get(), 10, "downstream write landed: a=1 → b=10");
        drop(first);
    }

    #[test]
    fn nested_create_effect_gets_its_own_frame() {
        let _serial = global_state_lock();
        let a = static_signal(0u32);
        let b = static_signal(0u32);
        let outer = counter();
        let inner = counter();
        let created = Arc::new(AtomicBool::new(false));

        let (o2, i2, cr) = (outer.clone(), inner.clone(), created.clone());
        let (a2, b2) = (a.clone(), b.clone());
        let _outer = create_effect(move || {
            o2.fetch_add(1, Ordering::SeqCst);
            a2.get();
            if !cr.swap(true, Ordering::SeqCst) {
                let (b3, i3) = (b2.clone(), i2.clone());
                let _inner = create_effect(move || {
                    i3.fetch_add(1, Ordering::SeqCst);
                    b3.get();
                });
            }
        });
        assert_eq!(runs(&outer), 1);
        assert_eq!(runs(&inner), 1, "inner effect ran its initial run");

        b.set(1);
        assert_eq!(runs(&inner), 2, "inner tracks its own dependency");
        assert_eq!(runs(&outer), 1, "outer did not adopt the inner read");

        a.set(1);
        assert_eq!(runs(&outer), 2);
        assert_eq!(runs(&inner), 2);
    }

    #[test]
    fn watch_does_not_fire_for_initial_value() {
        let _serial = global_state_lock();
        let source = static_signal(1u32);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (s2, seen2) = (source.clone(), seen.clone());
        let _handle = watch(&s2, move |value: &u32| {
            seen2.lock().unwrap().push(*value);
        });
        assert!(
            seen.lock().unwrap().is_empty(),
            "no fire for the initial value"
        );
        source.set(2);
        source.set(3);
        assert_eq!(*seen.lock().unwrap(), vec![2, 3]);
    }

    #[test]
    fn watch_fires_on_equal_write_but_not_after_set_if_eq_equal() {
        let _serial = global_state_lock();
        let source = static_signal("v1".to_string());
        let count = counter();
        let (s2, c2) = (source.clone(), count.clone());
        let _handle = watch(&s2, move |_value: &String| {
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&count), 0);

        source.set("v1".to_string()); // equal value, plain set: notifies
        assert_eq!(runs(&count), 1, "watch is write-driven (documented)");

        assert!(!source.set_if_eq("v1".to_string()), "equal write skipped");
        assert_eq!(runs(&count), 1, "set_if_eq equal write does not notify");

        assert!(source.set_if_eq("v2".to_string()));
        assert_eq!(runs(&count), 2);
    }

    #[test]
    fn watch_over_notifier_signal() {
        let _serial = global_state_lock();
        let source = crate::NotifierSignal::new(0u32, || {});
        let seen = counter();
        let seen2 = seen.clone();
        let _handle = watch(&source, move |value: &u32| {
            seen2.store(*value as usize, Ordering::SeqCst);
        });
        source.set(5);
        assert_eq!(seen.load(Ordering::SeqCst), 5);
        source.update(|v| *v += 2);
        assert_eq!(seen.load(Ordering::SeqCst), 7);
    }

    #[test]
    fn effect_body_panic_propagates_and_runtime_survives() {
        let _serial = global_state_lock();
        let source = static_signal(0u32);
        let source_in_panic = source.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _handle = create_effect(move || {
                source_in_panic.get();
                panic!("boom inside effect");
            });
        }));
        assert!(result.is_err(), "initial-run panic propagates");

        // The runtime recovers: a fresh effect works normally afterwards.
        let recovered = counter();
        let recovered2 = recovered.clone();
        let source2 = source.clone();
        let _handle = create_effect(move || {
            source2.get();
            recovered2.fetch_add(1, Ordering::SeqCst);
        });
        source.set(1);
        assert_eq!(runs(&recovered), 2);
    }

    #[test]
    fn unobserved_mutation_skips_subscriber_lock() {
        let _serial = global_state_lock();
        // No effects exist in this test: `notify` must short-circuit on the
        // live-effect counter (LIVE_EFFECTS == 0). This test simply asserts
        // the observable behaviour is unaffected.
        let source = static_signal(0u32);
        source.set(1);
        assert_eq!(source.get(), 1);
    }

    // -- Unwind safety ------------------------------------------------------
    //
    // A panicking effect body must leave the *thread's* tracking state
    // (TRACKING gate + FRAMES stack) exactly as it found it, and a failed
    // `create_effect` must leave no trace in the runtime's global registry.
    // These assertions read the module thread-locals and the global counters
    // directly; the project runner (cargo-nextest) executes every test in
    // its own process, so the globals are per-test there. Under a plain
    // `cargo test` the assertions still hold because each test body creates
    // and reaps its own effects around the observed window.

    #[test]
    fn panic_in_body_leaves_tracking_state_clean() {
        let _serial = global_state_lock();
        let source = static_signal(0u32);
        let armed = Arc::new(AtomicBool::new(false));
        let (s2, armed2) = (source.clone(), armed.clone());
        let _handle = create_effect(move || {
            s2.get();
            assert!(!armed2.swap(true, Ordering::SeqCst), "boom on the re-run");
        });
        assert!(!TRACKING.with(Cell::get), "idle between runs");
        assert!(FRAMES.with(|frames| frames.borrow().is_empty()));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            source.set(1); // triggers the panicking re-run
        }));
        assert!(result.is_err(), "the re-run panic propagates to the caller");

        assert!(
            !TRACKING.with(Cell::get),
            "a panicking body must not leave the tracking gate stuck on"
        );
        assert!(
            FRAMES.with(|frames| frames.borrow().is_empty()),
            "a panicking body must not leave a stale frame behind"
        );
    }

    #[test]
    fn panic_in_body_leaves_no_frame_growth_for_plain_reads() {
        let _serial = global_state_lock();
        let source = static_signal(0u32);
        let run_count = Arc::new(AtomicUsize::new(0));
        let (s2, rc2) = (source.clone(), run_count.clone());
        let handle = create_effect(move || {
            s2.get();
            // Panic exactly once (on the first re-run); later re-runs are
            // well-behaved so the effect stays a harmless neighbour.
            assert!(
                rc2.fetch_add(1, Ordering::SeqCst) != 1,
                "boom on the re-run"
            );
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            source.set(1);
        }));
        assert!(result.is_err());
        handle.dispose(); // the documented escape hatch after a body panic

        // Tracked reads outside any effect run pay one thread-local flag
        // check and must not accumulate state anywhere: the thread recovered.
        for _ in 0..1_000 {
            source.get();
        }
        assert!(FRAMES.with(|frames| frames.borrow().is_empty()));
        assert!(!TRACKING.with(Cell::get));

        // And a fresh effect on the recovered thread still tracks correctly.
        let recovered = counter();
        let (s3, c2) = (source.clone(), recovered.clone());
        let fresh = create_effect(move || {
            s3.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        source.set(2);
        assert_eq!(runs(&recovered), 2, "fresh effect tracks after recovery");
        fresh.dispose();
    }

    #[test]
    fn initial_run_panic_releases_the_zero_cost_gate() {
        let _serial = global_state_lock();
        let before = LIVE_EFFECTS.load(Ordering::SeqCst);
        let source = static_signal(0u32);
        let source2 = source.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_effect(move || {
                source2.get();
                panic!("boom inside the initial run");
            });
        }));
        assert!(result.is_err());

        assert_eq!(
            LIVE_EFFECTS.load(Ordering::SeqCst),
            before,
            "a failed creation must not keep the zero-cost gate disabled"
        );
    }

    #[test]
    fn initial_run_panic_does_not_leak_the_registry() {
        let _serial = global_state_lock();
        let before = registry_len();
        let source = static_signal(0u32);
        let source2 = source.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_effect(move || {
                source2.get();
                panic!("boom inside the initial run");
            });
        }));
        assert!(result.is_err());

        assert_eq!(
            registry_len(),
            before,
            "a failed creation must not leave a node in the registry"
        );
        assert!(registry_invariant_holds());
    }

    #[test]
    fn registry_slots_stay_consistent_through_dispose() {
        let _serial = global_state_lock();
        let handles: Vec<EffectHandle> = (0..8)
            .map(|_| {
                let source = static_signal(0u32);
                let s2 = source.clone();
                create_effect(move || {
                    s2.get();
                })
            })
            .collect();
        let before = registry_len();

        handles[3].dispose(); // interior removal (swap_remove moves the last)
        assert!(registry_invariant_holds(), "after interior dispose");
        handles[0].dispose(); // head removal
        assert!(registry_invariant_holds(), "after head dispose");
        handles[7].dispose(); // tail removal
        assert!(registry_invariant_holds(), "after tail dispose");

        // Fresh creations reuse the reclaimed capacity with valid slots.
        let extra = {
            let source = static_signal(0u32);
            let s2 = source.clone();
            create_effect(move || {
                s2.get();
            })
        };
        assert!(registry_invariant_holds(), "after re-creation");
        extra.dispose();
        assert!(registry_invariant_holds(), "after re-dispose");

        // Eight creations, then one interior + one head + one tail disposal
        // and one re-created + re-disposed effect: the registry holds exactly
        // the still-live nodes.
        assert_eq!(registry_len(), before - 3);
    }
}
