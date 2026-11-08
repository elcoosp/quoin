//! Floem adapter for quoin — reactive signals and async executors.
//!
//! This crate implements [`ReactiveContext`] for the [Floem](https://github.com/lapce/floem)
//! UI framework, enabling framework-agnostic hooks and state to run inside
//! Floem views.
//!
//! # Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`FloemContext`] | A zero-sized context. Creates signals via Floem's `RwSignal`. |
//! | [`FloemSignal<T>`] | A `Clone + Send + Sync` signal (for `T: Send`) backed by a shared `Mutex<T>` plus a version counter. |
//! | [`FloemExecutor`] | Spawns futures on worker threads with cooperative abort support. |
//! | [`FloemJoinHandle<T>`] | Shared oneshot handle (abort / detach / abort-on-drop) from `quoin-core`. |
//!
//! # Creating a Context
//!
//! ```ignore
//! fn app_view() -> impl IntoView {
//!     let ctx = FloemContext::new();
//!     let count = ctx.create_signal(0u32);
//!     // ...
//! }
//! ```
//!
//! # Signal Threading Model
//!
//! A [`FloemSignal`] keeps the authoritative value in a `Mutex<T>` and drives
//! floem's reactivity through a `RwSignal<u32>` version counter:
//!
//! - For `T: Send` (every real application state), `get`/`set`/`update` work
//!   from **any** thread. A write from a pool thread bumps the version;
//!   floem's [`floem_reactive`] runtime enqueues the resulting effects and
//!   runs them on the UI thread (its documented `SyncStorage` behavior), so
//!   background `set` calls never panic and never run view code off-thread.
//! - For `T: !Send` the signal type is single-thread-capable only (the
//!   compiler rejects sharing it), which is strictly sounder than the
//!   previous `SendWrapper` layer whose cross-thread dereference panicked.
//!
//! Reads track the version counter, so macro-generated views re-run exactly
//! when a signal they read changes — no double bookkeeping.
//!
//! # Global State (Thread-Local Store, Shared Values)
//!
//! `provide_global` and `use_global` use a **thread-local** type-map (same pattern
//! as the GPUI adapter). Globals are only visible on the registering thread.
//!
//! The stored value is the Floem `RwSignal` itself, so every `use_global`
//! consumer shares one reactive cell: mutations propagate between consumers
//! and drive Floem reactivity (unified with the Leptos adapter semantics).
//!
//! The adapter also implements [`quoin_core::LocalExecutor`]: `spawn_local`
//! queues `!Send` futures on the current thread (native), driven by
//! `quoin_core::local::run_local_tasks()`.

pub mod effect;
pub mod keys;
pub mod theme;

use floem_reactive::{SignalGet, SignalTrack, SignalUpdate};
use quoin_core::{DepHandle, Executor, ReactiveContext, Signal, TrackedSignal};
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// The storage flavor used by every signal the quoin Floem adapter hands out.
///
/// Floem's reactive primitives are generic over their storage: `UnsyncStorage`
/// (the default) is bound to the creating thread, `SyncStorage` is `Send +
/// Sync`. Quoin's [`Signal`] contract requires `Send + Sync` handles, so all
/// adapter signals opt into `SyncStorage` (`floem_reactive`'s `SyncRwSignal`);
/// values that are not themselves `Send + Sync` stay safe behind
/// [`SendWrapper`], matching the threading model documented on
/// [`FloemSignal`].
type QuoinRwSignal<T> = std::sync::Arc<FloemSignalInner<T>>;

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone, Default)]
/// Floem reactive context.
pub struct FloemContext;

impl FloemContext {
    /// Creates a fresh context backed by the shared floem update handle.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ReactiveContext for FloemContext {
    type Signal<T: Clone + Send + 'static> = FloemSignal<T>;
    type Executor = FloemExecutor;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        FloemSignal::new(initial)
    }

    fn executor(&self) -> Self::Executor {
        FloemExecutor
    }

    fn request_update(&self) {
        // Floem's reactivity is automatic.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        // Store the shared signal cell itself so every consumer shares the
        // same storage: mutations propagate between consumers (matching the
        // Leptos adapter; previously each use_global returned an independent
        // copy).
        GLOBAL_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(TypeId::of::<T>(), Box::new(FloemSignal::shared(value)));
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<FloemSignal<T>>())
                .cloned()
        })
    }
}

/// The shared storage behind a [`FloemSignal`]: the authoritative value plus
/// the version counter that drives floem's reactivity and the core effect
/// graph cell that portable effects subscribe to.
struct FloemSignalInner<T> {
    value: Mutex<T>,
    version: floem_reactive::SyncRwSignal<u32>,
    dep: DepHandle,
}

#[derive(Clone)]
/// Floem-backed reactive signal.
///
/// The value lives in a shared [`Mutex`]; floem reactivity is driven by a
/// `RwSignal<u32>` version counter that every read tracks. Writes from any
/// thread bump the version — floem's `SyncStorage` runtime enqueues the
/// dependent effects and runs them on the UI thread, so background mutations
/// are both panic-free and correctly scheduled. Requires `T: Send` for the
/// handle to be shareable (enforced by the type system).
pub struct FloemSignal<T: Clone + 'static> {
    inner: QuoinRwSignal<T>,
}

impl<T: Clone + 'static> FloemSignal<T> {
    fn new(initial: T) -> Self {
        Self::shared(initial)
    }

    /// Builds a signal whose storage is shared by every clone (the shape
    /// `provide_global` uses so consumers see one reactive cell).
    fn shared(initial: T) -> Self {
        Self {
            inner: Arc::new(FloemSignalInner {
                value: Mutex::new(initial),
                version: floem_reactive::SyncRwSignal::new_sync(0),
                dep: DepHandle::new(),
            }),
        }
    }

    /// Bumps the version counter, notifying floem subscribers and the core
    /// effect graph (portable effects re-run on the write, FIFO and
    /// batch-coalesced like every other tracked signal).
    fn bump(&self) {
        let next = self.inner.version.get_untracked().wrapping_add(1);
        self.inner.version.set(next);
        self.inner.dep.notify();
    }
}

impl<T: Clone + Send + 'static> Signal<T> for FloemSignal<T> {
    fn get(&self) -> T {
        // Tracking the version is what registers this read with floem's
        // reactive graph, so views re-run on the next write; the dep cell
        // registers the same read with the core effect graph.
        self.inner.dep.track_read();
        self.inner.version.track();
        self.inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Reads the current value through `f`.
    ///
    /// # Deviation from the generic `Signal::with` contract
    ///
    /// The generic contract promises clone-free access, but the authoritative
    /// value sits behind a non-reentrant `Mutex`: holding the guard across
    /// `f` would deadlock any *nested* access to the same signal (a `get()`
    /// or `with()` inside the closure, as the `signal_nested_with`
    /// conformance check exercises). The adapter clones instead; the fast
    /// path used by macro-generated views is `get()`, which clones in every
    /// adapter, so generated UI code pays no extra cost on this backend.
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        // `get` tracks the dep cell (and the floem version counter).
        let value = self.get();
        f(&value)
    }

    fn set(&self, value: T) {
        *self
            .inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
        self.bump();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self
            .inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut guard);
        drop(guard);
        self.bump();
    }
}

impl<T: Clone + Send + 'static> TrackedSignal<T> for FloemSignal<T> {
    fn dep_handle(&self) -> DepHandle {
        self.inner.dep.clone()
    }
}

/// Floem async executor.
///
/// Futures run on dedicated worker threads via `futures::executor::block_on`
/// and are wrapped in `futures::future::Abortable`, so `JoinHandle::abort`
/// stops them at their next suspension point.
#[derive(Clone)]
pub struct FloemExecutor;

impl Executor for FloemExecutor {
    type JoinHandle<T: Send + 'static> = FloemJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Shared implementation: the future is wrapped in an `Abortable` and
        // runs on the shared worker pool. See `quoin_core::spawn_abortable`.
        quoin_core::spawn_abortable(future)
    }
}

/// A handle to a task spawned by [`FloemExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
///
/// This is [`quoin_core::AbortJoinHandle`] under a historical name: the
/// handle lifecycle (await / abort / detach / abort-on-drop) is implemented
/// once in `quoin-core` and shared with the Leptos and Dioxus adapters.
pub type FloemJoinHandle<T> = quoin_core::AbortJoinHandle<T>;

impl quoin_core::LocalExecutor for FloemExecutor {
    type JoinHandle<T: 'static> = quoin_core::LocalTask<T>;

    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: std::future::Future + 'static,
        F::Output: 'static,
    {
        // On wasm the task runs on the browser main thread; on native it is
        // queued on the current thread (drive with
        // `quoin_core::local::run_local_tasks()`).
        quoin_core::spawn_local_task(future)
    }
}

impl<T: Clone + std::fmt::Debug + Send> std::fmt::Debug for FloemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self
            .inner
            .value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_struct("FloemSignal")
            .field("value", &*value)
            // The dep cell is shared bookkeeping, not part of the value.
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `set` from a pool thread must not panic (the old `SendWrapper`
    /// storage panicked as soon as the UI thread read the value) and the
    /// written value must be visible from any thread afterwards.
    #[test]
    fn cross_thread_set_is_panic_free_and_visible() {
        let signal = FloemSignal::new(0u32);

        // Set on the creating thread first: the pool thread must be able to
        // READ a value that was written elsewhere without panicking either.
        signal.set(7);
        let pool_signal = signal.clone();
        let handle = std::thread::spawn(move || {
            assert_eq!(pool_signal.get(), 7);
            pool_signal.set(41);
            pool_signal.update(|v| *v += 1);
            pool_signal.get()
        });
        assert_eq!(handle.join().expect("pool thread must not panic"), 42);
        assert_eq!(signal.get(), 42);

        // `set_if_eq` keeps its single-lock contract across threads too.
        assert!(!signal.set_if_eq(42));
        assert!(signal.set_if_eq(43));
        assert_eq!(signal.get(), 43);
    }

    /// `with` sees the freshest value from a foreign thread (clone-based
    /// read; the nested-`with` conformance check covers the lock contract).
    #[test]
    fn with_reads_latest_value_cross_thread() {
        let signal = FloemSignal::new(String::from("init"));
        let handle = std::thread::spawn(move || signal.with(|v| v.clone() + "-worker"));
        assert_eq!(
            handle.join().expect("pool thread must not panic"),
            "init-worker"
        );
    }
}
