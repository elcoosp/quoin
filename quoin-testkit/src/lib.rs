//! # quoin-testkit — test framework-agnostic quoin code without a UI
//!
//! The whole point of quoin is that reactive business logic (hooks, stores,
//! view models) is written once against the [`ReactiveContext`] trait and can
//! be unit-tested without spinning up GPUI/Leptos/Dioxus/Xilem/Floem. This
//! crate ships the missing piece: concrete, dependency-light implementations
//! of the core traits designed for tests.
//!
//! ## What is in the box
//!
//! | Component | Purpose |
//! |-----------|---------|
//! | [`TestContext`] | Full `ReactiveContext` with update counting and a shared global store. |
//! | [`ImmediateExecutor`] | Runs futures **inline to completion** — fully deterministic, but blocking. |
//! | [`ThreadedExecutor`] | Runs each future on a dedicated thread — real concurrency, real abort semantics. |
//! | `TokioExecutor` *(feature `tokio`)* | Runs futures on a real tokio runtime. |
//! | [`UpdateRecorder`] | Assert how many times the UI was asked to re-render. |
//!
//! ## Choosing an executor
//!
//! - Use [`ImmediateExecutor`] for pure state-machine logic (reducers,
//!   derived pipelines): tests are synchronous and deterministic.
//! - Use [`ThreadedExecutor`] (or the tokio one) when the logic under test
//!   awaits timers/channels or when you exercise abort/detach semantics —
//!   `ImmediateExecutor` cannot model mid-flight cancellation because futures
//!   finish before the handle exists.
//!
//! ## Example
//!
//! ```
//! use quoin_testkit::{TestContext, ImmediateExecutor};
//! use quoin_core::prelude::*;
//!
//! // Framework-agnostic hook under test:
//! fn use_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
//!     cx.create_signal(0u32)
//! }
//!
//! let mut cx = TestContext::with_executor(ImmediateExecutor);
//! let count = use_counter(&cx);
//! count.set(41);
//! count.update(|v| *v += 1);
//! assert_eq!(count.get(), 42);
//!
//! // Update plumbing is observable:
//! assert_eq!(cx.updates().count(), 2, "set + update each requested a re-render");
//! ```

use quoin_core::batch;
use quoin_core::effect::DepHandle;
use quoin_core::executor::{Executor, JoinHandle};
use quoin_core::local::{LocalExecutor, LocalJoinHandle};
use quoin_core::reactive::ReactiveContext;
use quoin_core::signal::Signal;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

// ---------------------------------------------------------------------------
// TestSignal
// ---------------------------------------------------------------------------

/// The shared cell behind a [`TestSignal`]: value plus effect dependency
/// handle, so every handle over the same cell (notably via `use_global`)
/// notifies the same subscribers.
struct TestCell<T> {
    value: RwLock<T>,
    dep: DepHandle,
}

/// A standalone, thread-safe signal backed by a shared cell.
///
/// Mutations bump the owning [`TestContext`]'s update counter, mirroring the
/// "pull-based adapter" behaviour (GPUI/Xilem): every write asks for a
/// re-render unless suppressed by [`quoin_core::batch`]. Reads inside
/// [`create_effect`](quoin_core::create_effect) register dependencies and
/// mutations re-run effects — the same effect semantics as the production
/// pull-based adapters.
#[derive(Clone)]
pub struct TestSignal<T> {
    cell: Arc<TestCell<T>>,
    updates: Arc<AtomicUsize>,
}

impl<T> TestSignal<T> {
    fn new(initial: T, updates: Arc<AtomicUsize>) -> Self {
        Self {
            cell: Arc::new(TestCell {
                value: RwLock::new(initial),
                dep: DepHandle::new(),
            }),
            updates,
        }
    }

    fn record_update(&self) {
        if batch::should_notify() {
            self.updates.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl<T: std::fmt::Debug + Clone + 'static> std::fmt::Debug for TestSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The update counter is context-shared bookkeeping, not part of the
        // value; expose it as non-exhaustive instead of a field.
        f.debug_struct("TestSignal")
            .field("value", &*self.cell.value.read().unwrap())
            .finish_non_exhaustive()
    }
}

impl<T: Clone + 'static> Signal<T> for TestSignal<T> {
    fn get(&self) -> T {
        self.cell.dep.track_read();
        self.cell.value.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.cell.dep.track_read();
        f(&self.cell.value.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.cell.value.write().unwrap() = value;
        self.cell.dep.notify();
        self.record_update();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.cell.value.write().unwrap());
        self.cell.dep.notify();
        self.record_update();
    }

    fn set_if_eq(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        let mut guard = self.cell.value.write().unwrap();
        if *guard == value {
            return false;
        }
        *guard = value;
        drop(guard);
        self.cell.dep.notify();
        self.record_update();
        true
    }
}

impl<T: Clone + 'static> quoin_core::TrackedSignal<T> for TestSignal<T> {
    fn dep_handle(&self) -> quoin_core::DepHandle {
        self.cell.dep.clone()
    }
}

// ---------------------------------------------------------------------------
// UpdateRecorder / TestContext
// ---------------------------------------------------------------------------

/// Counter for the `request_update` calls issued through a [`TestContext`].
///
/// Clone shares the same underlying counter.
#[derive(Clone, Default)]
pub struct UpdateRecorder {
    count: Arc<AtomicUsize>,
}

impl UpdateRecorder {
    /// Creates a zeroed recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of update requests observed so far.
    #[must_use]
    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    /// Resets the counter to zero.
    pub fn reset(&self) {
        self.count.store(0, Ordering::SeqCst);
    }

    /// Asserts the counter equals `expected` (panics with a helpful message).
    ///
    /// # Panics
    ///
    /// Panics when the observed count differs from `expected`.
    #[track_caller]
    pub fn assert_count(&self, expected: usize) {
        let actual = self.count();
        assert_eq!(
            actual, expected,
            "expected {expected} update request(s), observed {actual}"
        );
    }
}

/// The default executor used by [`TestContext::new`].
///
/// Spawns each future on its own thread and blocks a `futures` executor on
/// it. Deterministic enough for most tests while preserving abort semantics.
#[derive(Clone, Copy, Debug, Default)]
pub struct ThreadedExecutor;

/// Handle produced by [`ThreadedExecutor`].
pub struct ThreadedJoinHandle<T> {
    abort: futures::future::AbortHandle,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: Arc<AtomicBool>,
}

impl<T: Send + 'static> JoinHandle<T> for ThreadedJoinHandle<T> {
    fn abort(&self) {
        self.abort.abort();
    }

    fn detach(self) {
        self.detached.store(true, Ordering::SeqCst);
    }
}

impl<T> Drop for ThreadedJoinHandle<T> {
    fn drop(&mut self) {
        if !self.detached.load(Ordering::SeqCst) {
            self.abort.abort();
        }
    }
}

impl<T: Send + 'static> Future for ThreadedJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.rx.as_mut().map_or_else(
            || std::task::Poll::Ready(Err(futures::channel::oneshot::Canceled)),
            |rx| Pin::new(rx).poll(cx),
        )
    }
}

impl Executor for ThreadedExecutor {
    type JoinHandle<T: Send + 'static> = ThreadedJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let (abort, registration) = futures::future::AbortHandle::new_pair();
        let detached = Arc::new(AtomicBool::new(false));
        std::thread::spawn(move || {
            let task = async move {
                let value = future.await;
                let _ = tx.send(value);
            };
            let _ =
                futures::executor::block_on(futures::future::Abortable::new(task, registration));
        });
        ThreadedJoinHandle {
            abort,
            rx: Some(rx),
            detached,
        }
    }
}

impl ThreadedExecutor {
    /// Sleep helper, useful in tests that need real time to pass.
    pub async fn sleep(&self, duration: std::time::Duration) {
        futures_timer::Delay::new(duration).await;
    }
}

/// Runs every future **inline to completion** on the calling thread.
///
/// `spawn(fut)` simply drives `fut` to completion and returns a handle that
/// already holds the result. Consequences:
///
/// - fully deterministic, no threads, no flakiness;
/// - a future that awaits external events (timers, channels fed elsewhere)
///   will **hang** — use [`ThreadedExecutor`] for those;
/// - aborting after spawn is a no-op (the task already finished).
#[derive(Clone, Copy, Debug, Default)]
pub struct ImmediateExecutor;

/// Handle produced by [`ImmediateExecutor`]; the result is already available.
pub struct ImmediateJoinHandle<T> {
    result: Mutex<Option<T>>,
}

impl<T> ImmediateJoinHandle<T> {
    /// Takes the result, panicking if it was already taken.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (a prior panic while
    /// running the spawned future inline).
    pub fn take_result(&self) -> Option<T> {
        self.result.lock().unwrap().take()
    }
}

impl<T: Send> JoinHandle<T> for ImmediateJoinHandle<T> {
    fn abort(&self) {
        // Already completed.
    }

    fn detach(self)
    where
        Self: Sized,
    {
        // Already completed.
    }
}

impl<T> Future for ImmediateJoinHandle<T> {
    type Output = T;

    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(
            self.get_mut()
                .result
                .get_mut()
                .unwrap()
                .take()
                .expect("ImmediateJoinHandle polled after completion"),
        )
    }
}

impl<T> Unpin for ImmediateJoinHandle<T> {}

impl Executor for ImmediateExecutor {
    type JoinHandle<T: Send + 'static> = ImmediateJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        ImmediateJoinHandle {
            result: Mutex::new(Some(quoin_core::pool::block_on(future))),
        }
    }
}

/// Tokio-backed executor (enable the `tokio` feature).
#[cfg(feature = "tokio")]
#[derive(Clone, Debug, Default)]
pub struct TokioExecutor {
    runtime: Option<Arc<tokio::runtime::Runtime>>,
}

#[cfg(feature = "tokio")]
impl TokioExecutor {
    /// Creates an executor bound to a fresh multi-thread runtime.
    ///
    /// Prefer [`TokioExecutor::global`] inside `#[tokio::test]` so tasks run
    /// on the test's own runtime (and time is controlled by `pause()`).
    ///
    /// # Panics
    ///
    /// Panics if the tokio multi-thread runtime cannot be built (e.g. the
    /// process exhausted the resources needed to spawn its worker threads);
    /// the underlying `Builder::build` error message is preserved.
    #[must_use]
    pub fn new() -> Self {
        Self {
            runtime: Some(Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build tokio runtime"),
            )),
        }
    }

    /// Uses the ambient tokio runtime (`#[tokio::test]`, `#[tokio::main]`).
    #[must_use]
    pub fn global() -> Self {
        Self { runtime: None }
    }
}

#[cfg(feature = "tokio")]
/// Join handle for tasks spawned by [`TokioExecutor`].
///
/// Awaiting resolves with the task's output or `Err(Canceled)` when aborted;
/// dropping the handle without detaching aborts the task (structured
/// concurrency, matching every other executor in this crate).
pub struct TokioJoinHandle<T> {
    inner: Option<tokio::task::JoinHandle<()>>,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: Arc<AtomicBool>,
}

#[cfg(feature = "tokio")]
impl<T: Send + 'static> JoinHandle<T> for TokioJoinHandle<T> {
    fn abort(&self) {
        if let Some(handle) = &self.inner {
            handle.abort();
        }
    }

    fn detach(self) {
        // tokio's `JoinHandle` never cancels its task on drop, so there is
        // nothing to release: just mark this handle as detached so `Drop`
        // does not abort the task.
        self.detached.store(true, Ordering::SeqCst);
    }
}

#[cfg(feature = "tokio")]
impl<T> Drop for TokioJoinHandle<T> {
    fn drop(&mut self) {
        if !self.detached.load(Ordering::SeqCst)
            && let Some(handle) = self.inner.take()
        {
            handle.abort();
        }
    }
}

#[cfg(feature = "tokio")]
impl<T: Send + 'static> Future for TokioJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.rx.as_mut() {
            Some(rx) => Pin::new(rx).poll(cx),
            None => std::task::Poll::Ready(Err(futures::channel::oneshot::Canceled)),
        }
    }
}

#[cfg(feature = "tokio")]
impl Executor for TokioExecutor {
    type JoinHandle<T: Send + 'static> = TokioJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let inner = match &self.runtime {
            Some(rt) => rt.spawn(async move {
                let _ = tx.send(future.await);
            }),
            None => tokio::spawn(async move {
                let _ = tx.send(future.await);
            }),
        };
        TokioJoinHandle {
            inner: Some(inner),
            rx: Some(rx),
            detached: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Delegates to `tokio::time::sleep` so paused-time tests
    /// (`#[tokio::test(start_paused = true)]` + `advance`) control every
    /// timer deterministically.
    fn sleep(&self, duration: std::time::Duration) -> quoin_core::SleepFuture {
        Box::pin(tokio::time::sleep(duration))
    }
}

// ---------------------------------------------------------------------------
// Local (non-Send) execution
// ---------------------------------------------------------------------------

/// Handle for local tasks spawned on [`ThreadedExecutor`].
///
/// Delegates to `quoin_core::LocalTask`: tasks are queued on the current
/// thread (deterministic) and driven by
/// `quoin_core::local::run_local_tasks()`.
pub struct LocalThreadedJoinHandle<T> {
    task: quoin_core::LocalTask<T>,
}

impl<T: 'static> LocalJoinHandle<T> for LocalThreadedJoinHandle<T> {
    fn abort(&self) {
        quoin_core::LocalJoinHandle::abort(&self.task);
    }

    fn detach(self) {
        quoin_core::LocalJoinHandle::detach(self.task);
    }
}

impl<T: 'static> Future for LocalThreadedJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(&mut self.task).poll(cx)
    }
}

impl LocalExecutor for ThreadedExecutor {
    type JoinHandle<T: 'static> = LocalThreadedJoinHandle<T>;

    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        // Deterministic: the task is queued on the current thread and runs
        // when `quoin_core::local::run_local_tasks()` is called. This keeps
        // local-task tests fully reproducible (no threads, no races).
        let task = quoin_core::spawn_local_task(future);
        LocalThreadedJoinHandle { task }
    }
}

// ---------------------------------------------------------------------------
// TestContext
// ---------------------------------------------------------------------------

thread_local! {
    static LOCAL_GLOBALS: RefCell<HashMap<TypeId, Rc<dyn std::any::Any>>> =
        RefCell::new(HashMap::new());
}

/// A full-featured [`ReactiveContext`] for testing framework-agnostic code.
///
/// - `create_signal` returns [`TestSignal`]s whose mutations count as update
///   requests (unless inside a [`quoin_core::batch`]).
/// - `request_update` increments the same counter.
/// - `provide_global` / `use_global` use a **shared** store: mutations through
///   one `use_global` signal are visible through another, matching the
///   unified semantics of the production adapters.
/// - The executor is generic: pick [`ImmediateExecutor`],
///   [`ThreadedExecutor`], or (with the `tokio` feature) `TokioExecutor`.
#[derive(Clone)]
pub struct TestContext<E: Executor = ThreadedExecutor> {
    updates: UpdateRecorder,
    executor: E,
    globals: Arc<Mutex<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,
}

impl TestContext<ThreadedExecutor> {
    /// Creates a context with the default [`ThreadedExecutor`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_executor(ThreadedExecutor)
    }
}

impl Default for TestContext<ThreadedExecutor> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Executor> TestContext<E> {
    /// Creates a context with a specific executor.
    #[must_use]
    pub fn with_executor(executor: E) -> Self {
        Self {
            updates: UpdateRecorder::new(),
            executor,
            globals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The update recorder: inspect and assert on re-render requests.
    #[must_use]
    pub const fn updates(&self) -> &UpdateRecorder {
        &self.updates
    }

    /// The executor instance used by this context.
    #[must_use]
    pub const fn executor_handle(&self) -> &E {
        &self.executor
    }
}

impl<E: Executor> ReactiveContext for TestContext<E> {
    type Signal<T: Clone + Send + 'static> = TestSignal<T>;
    type Executor = E;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        TestSignal::new(initial, self.updates.count.clone())
    }

    fn executor(&self) -> Self::Executor {
        self.executor.clone()
    }

    fn request_update(&self) {
        self.updates.count.fetch_add(1, Ordering::SeqCst);
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        // Store the value inside a shared cell so every `use_global`
        // consumer observes the same mutations (matching the unified
        // production-adapter semantics). The effect dependency handle lives
        // inside the cell, so every handle handed out by `use_global`
        // shares one subscription list.
        self.globals.lock().unwrap().insert(
            TypeId::of::<SharedGlobal<T>>(),
            Box::new(Arc::new(TestCell {
                value: RwLock::new(value),
                dep: DepHandle::new(),
            })),
        );
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        let cell = self
            .globals
            .lock()
            .unwrap()
            .get(&TypeId::of::<SharedGlobal<T>>())?
            .downcast_ref::<Arc<TestCell<T>>>()?
            .clone();
        Some(TestSignal {
            cell,
            updates: self.updates.count.clone(),
        })
    }
}

/// Marker key under which shared global cells are stored, so repeated
/// `use_global` calls retrieve the same cell instead of fresh copies.
struct SharedGlobal<T> {
    _marker: std::marker::PhantomData<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_core::{ReactiveContextExt, SignalExt};

    #[test]
    fn signal_records_updates() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let s = cx.create_signal(0u32);
        cx.updates().assert_count(0);
        s.set(1);
        cx.updates().assert_count(1);
        s.update(|v| *v += 1);
        cx.updates().assert_count(2);
        let _ = s.get(); // reads do not count
        cx.updates().assert_count(2);
        s.set(s.get()); // writes count even when the value is unchanged
        cx.updates().assert_count(3);
    }

    #[test]
    fn batch_collapses_updates_to_one() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let a = cx.create_signal(0u32);
        let b = cx.create_signal(0u32);
        cx.batch(|| {
            a.set(1);
            b.set(2);
            a.set(3);
        });
        cx.updates().assert_count(1);
        assert_eq!(a.get(), 3);
        assert_eq!(b.get(), 2);
    }

    #[test]
    fn set_eq_skips_equal_writes() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let s = cx.create_signal("x".to_string());
        assert!(!s.set_eq("x".to_string()));
        cx.updates().assert_count(0);
        assert!(s.set_eq("y".to_string()));
        cx.updates().assert_count(1);
    }

    #[test]
    fn request_update_counts_directly() {
        let cx = TestContext::new();
        cx.request_update();
        cx.request_update();
        cx.updates().assert_count(2);
        cx.updates().reset();
        cx.updates().assert_count(0);
    }

    #[test]
    fn globals_are_shared_between_consumers() {
        let cx = TestContext::new();
        cx.provide_global(10u32);
        let sig1 = cx.use_global::<u32>().expect("global present");
        let sig2 = cx.use_global::<u32>().expect("global present");
        sig1.set(55);
        assert_eq!(sig2.get(), 55, "use_global must share state");
        assert_eq!(sig1.get(), 55);
    }

    #[test]
    fn missing_global_returns_none() {
        let cx = TestContext::new();
        assert!(cx.use_global::<String>().is_none());
    }

    #[test]
    fn immediate_executor_completes_inline() {
        let executor = ImmediateExecutor;
        let handle = executor.spawn(async { 21 * 2 });
        assert_eq!(handle.result.lock().unwrap().as_ref().copied(), Some(42));
    }

    #[test]
    fn threaded_executor_provides_result() {
        let executor = ThreadedExecutor;
        let handle = executor.spawn(async { "hi".to_string() });
        let value = futures::executor::block_on(handle).expect("not aborted");
        assert_eq!(value, "hi");
    }

    #[test]
    fn threaded_executor_abort_cancels() {
        let executor = ThreadedExecutor;
        let (tx, _rx) = futures::channel::oneshot::channel::<()>();
        let (gate_tx, gate_rx) = futures::channel::oneshot::channel::<()>();
        let handle = executor.spawn(async move {
            let _ = gate_rx.await; // wait to be released
            let _ = tx.send(());
        });
        handle.abort();
        let _ = gate_tx.send(());
        let result = futures::executor::block_on(handle);
        assert!(result.is_err(), "aborted task must resolve with Canceled");
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn tokio_executor_runs_on_ambient_runtime() {
        let executor = TokioExecutor::global();
        let handle = executor.spawn(async { 5u32 });
        assert_eq!(handle.await.expect("not aborted"), 5);
    }

    #[test]
    fn update_global_and_use_global_or_init() {
        let cx = TestContext::new();
        assert!(cx.update_global::<u32, ()>(|_| {}).is_none());
        let sig = cx.use_global_or_init(|| 7u32);
        assert_eq!(sig.get(), 7);
        cx.update_global::<u32, ()>(|v| *v += 1);
        assert_eq!(sig.get(), 8, "update_global mutates the shared value");
    }

    #[test]
    fn globals_of_different_types_are_independent() {
        let cx = TestContext::new();
        cx.provide_global(1u32);
        cx.provide_global(String::from("one"));
        let n = cx.use_global::<u32>().expect("u32 global");
        let s = cx.use_global::<String>().expect("String global");
        n.set(2);
        s.update(|value| value.push_str("-x"));
        assert_eq!(cx.use_global::<u32>().unwrap().get(), 2);
        assert_eq!(cx.use_global::<String>().unwrap().get(), "one-x");
    }

    #[test]
    fn recorder_clone_shares_counter() {
        let cx = TestContext::new();
        let recorder = cx.updates().clone();
        recorder.reset();
        cx.request_update();
        assert_eq!(recorder.count(), 1, "clone shares the underlying counter");
        let sig = cx.create_signal(0u32);
        sig.set(1);
        assert_eq!(recorder.count(), 2);
    }

    #[test]
    fn test_signal_debug_shows_value() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let s = cx.create_signal(3u32);
        let debug = format!("{s:?}");
        assert!(debug.contains("value: 3"), "got: {debug}");
        assert!(debug.contains("TestSignal"));
        assert!(
            debug.contains(".."),
            "hidden bookkeeping fields are non-exhaustive"
        );
    }

    #[test]
    fn batch_suppression_is_scoped_to_calling_thread() {
        use quoin_core::batch::in_batch;
        assert!(!in_batch());
        let cx = TestContext::new();
        let s = cx.create_signal(0u32);
        cx.batch(|| {
            assert!(in_batch());
            s.set(1);
        });
        assert!(!in_batch(), "batch ends after with() returns");
        cx.updates().assert_count(1);
    }

    #[test]
    fn local_executor_queues_on_current_thread() {
        use quoin_core::local::{pending_local_tasks, run_local_tasks};
        let executor = ThreadedExecutor;
        let handle = quoin_core::LocalExecutor::spawn_local(&executor, async { 6 * 7 });
        // Deterministic: nothing runs until the queue is drained.
        assert_eq!(
            pending_local_tasks(),
            1,
            "task waits in the thread-local queue"
        );
        assert_eq!(run_local_tasks(), 1, "drain runs the queued task");
        let value = futures::executor::block_on(handle);
        assert_eq!(value, Ok(42));
    }

    #[test]
    fn threaded_executor_detach_runs_to_completion() {
        let executor = ThreadedExecutor;
        let flag = Arc::new(AtomicBool::new(false));
        let flag2 = flag.clone();
        let handle = executor.spawn(async move {
            std::thread::sleep(std::time::Duration::from_millis(20));
            flag2.store(true, Ordering::SeqCst);
        });
        handle.detach();
        for _ in 0..100 {
            if flag.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("detached task must run to completion");
    }

    #[test]
    fn dropping_threaded_handle_cancels_task() {
        let executor = ThreadedExecutor;
        let flag = Arc::new(AtomicBool::new(false));
        let flag2 = flag.clone();
        {
            let handle = executor.spawn(async move {
                // A real suspension point so the abort can interrupt it.
                futures_timer::Delay::new(std::time::Duration::from_millis(50)).await;
                flag2.store(true, Ordering::SeqCst);
            });
            drop(handle);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(
            !flag.load(Ordering::SeqCst),
            "dropping the handle without detaching cancels the task"
        );
    }

    // -----------------------------------------------------------------------
    // Core effects over TestContext signals
    // -----------------------------------------------------------------------

    #[test]
    fn test_signal_effect_reruns_on_mutation() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let count = cx.create_signal(0u32);
        let runs = Arc::new(AtomicUsize::new(0));
        let (count2, runs2) = (count.clone(), runs.clone());
        let _handle = quoin_core::create_effect(move || {
            let _value = count2.get();
            runs2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        count.set(1);
        assert_eq!(
            runs.load(Ordering::SeqCst),
            2,
            "TestSignal mutations re-run effects"
        );
        count.set(2);
        assert_eq!(runs.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn effect_rerun_does_not_inflate_update_counter() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let count = cx.create_signal(0u32);
        let (count2,) = (count.clone(),);
        let _handle = quoin_core::create_effect(move || {
            let _value = count2.get();
        });
        assert_eq!(
            cx.updates().count(),
            0,
            "the initial effect run is not a write"
        );
        count.set(1);
        // One write → one update request; the effect re-run it causes reads
        // only and must not look like another write to the recorder.
        assert_eq!(cx.updates().count(), 1);
    }

    #[test]
    fn watch_over_test_signal_sees_values() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let name = cx.create_signal("a".to_string());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (name2, seen2) = (name.clone(), seen.clone());
        let _handle = quoin_core::watch(&name2, move |value: &String| {
            seen2.lock().unwrap().push(value.clone());
        });
        name.set("b".to_string());
        name.set("c".to_string());
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn global_handles_share_one_effect_dependency() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        cx.provide_global(0u32);
        let first = cx.use_global::<u32>().expect("global provided");
        let second = cx.use_global::<u32>().expect("global provided");
        let runs = Arc::new(AtomicUsize::new(0));
        let (second2, runs2) = (second.clone(), runs.clone());
        let _handle = quoin_core::create_effect(move || {
            let _value = second2.get();
            runs2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        // Mutating through a *different* handle of the same global cell must
        // still reach the effect: the dependency lives in the shared cell.
        first.set(7);
        assert_eq!(
            runs.load(Ordering::SeqCst),
            2,
            "globals share one dependency cell"
        );
        assert_eq!(second.get(), 7);
    }
}
pub use quoin_core::local::LocalExecutor;
