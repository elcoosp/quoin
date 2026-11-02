//! Conformance test suite for `quoin` framework adapters.
//!
//! This crate provides a set of reusable test functions that verify an adapter's
//! `ReactiveContext` implementation meets the specification. Adapter crates use
//! the `define_conformance_tests!` macro to generate the actual test functions
//! with the appropriate test runner.
//!
//! Tests come in two flavours, and the macro picks the right wiring for each:
//!
//! - **Sync tests** (plain `fn`) cover purely reactive behaviour — signals,
//!   globals, derived values, batching. They run directly on the calling
//!   thread, no executor round-trip.
//! - **Async tests** (`async fn`) cover executor-backed behaviour — spawn,
//!   abort, detach, timed cancellation. They are wrapped in the harness's
//!   `block_on` (sync adapters) or an `#[gpui::test]` async body (GPUI).
//!
//! # Usage for non-GPUI adapters
//!
//! ```rust,ignore
//! use quoin_core::ReactiveContext;
//! use quoin_conformance::{define_conformance_tests, TestContextProvider};
//!
//! struct MyTestHarness { ... }
//! impl ReactiveContext for MyTestHarness { ... }
//! impl TestContextProvider for MyTestHarness {
//!     fn setup_context() -> Self { ... }
//! }
//!
//! define_conformance_tests!(sync, MyTestHarness);
//! ```
//!
//! # Usage for GPUI adapters
//!
//! ```rust,ignore
//! use gpui::TestAppContext;
//! use quoin_core::ReactiveContext;
//! use quoin_conformance::define_conformance_tests;
//!
//! struct TestHarness { context: GpuiContext }
//! impl TestHarness {
//!     fn new(cx: &mut TestAppContext) -> Self { ... }
//! }
//! impl ReactiveContext for TestHarness { ... }
//! define_conformance_tests!(gpui, TestHarness);
//! ```

use quoin_core::prelude::*;
use std::future::Future;

// -----------------------------------------------------------------------------
// Helper traits
// -----------------------------------------------------------------------------

/// How the conformance suite obtains a context and drives futures for a
/// given adapter.
///
/// Implement this for your test harness, then call
/// [`define_conformance_tests!`] to generate the suite.
pub trait TestContextProvider: ReactiveContext + Sized {
    /// Creates a fresh harness context for one test.
    fn setup_context() -> Self;

    /// Blocks the calling thread until `future` resolves.
    ///
    /// Defaults to `futures::executor::block_on`; override when the adapter
    /// needs a specific runtime (e.g. a tokio handle).
    fn block_on<F: Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }
}

// -----------------------------------------------------------------------------
// Core sync test functions
// -----------------------------------------------------------------------------

/// The conformance test functions.
///
/// Sync functions cover purely reactive behaviour; async functions exercise
/// the executor. [`define_conformance_tests!`] wires both flavours into the
/// right harness automatically — call it instead of using this module by hand.
pub mod tests {
    use quoin_core::{
        CancellationToken, Executor, JoinHandle, LocalExecutor, LocalJoinHandle, ReactiveContext,
        ReactiveContextExt, Signal, SignalExt,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    /// A freshly created signal returns its initial value.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn create_signal_initial_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        assert_eq!(signal.get(), 42);
    }

    /// `with` lends a reference to the current value.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_with_borrowing<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal("hello".to_string());
        signal.with(|value| {
            assert_eq!(value, "hello");
        });
    }

    /// Signals are `Clone`; the copy observes the same state.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_clone_and_copy<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(100u32);
        let copy = signal;
        assert_eq!(copy.get(), 100);
    }

    // --- Mutation tests ---

    /// `set` writes the value through to the signal.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_set_updates_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(0u32);
        signal.set(42);
        assert_eq!(signal.get(), 42);
    }

    /// `update` applies in-place mutations sequentially.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_update_modifies_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(0u32);
        signal.update(|v| *v += 5);
        signal.update(|v| *v *= 2);
        assert_eq!(signal.get(), 10);
    }

    /// Mutations are observable through later `with` reads.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_mutation_observable_via_with<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(String::from("hello"));
        signal.update(|s| s.push_str(" world"));
        signal.with(|s| assert_eq!(s, "hello world"));
    }

    // --- Edge-case tests ---

    /// Verify that a clone of a signal shares the same state
    /// Clones share state: a write via the clone is visible in the original.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_clone_then_mutate<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        let clone = signal.clone();
        clone.set(99);
        assert_eq!(signal.get(), 99);
    }

    /// Ensure nested `with` calls do not deadlock or panic
    /// Nested `with` borrows neither deadlock nor panic.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_nested_with<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        signal.with(|outer| {
            signal.with(|inner| {
                assert_eq!(*outer, *inner);
            });
        });
    }

    /// Stress test with a large value (10k elements)
    /// Large payloads (10k elements) round-trip intact.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn signal_large_value<C: ReactiveContext>(cx: &C) {
        let large: Vec<u32> = (0..10_000).collect();
        let signal = cx.create_signal(large.clone());
        assert_eq!(signal.get(), large);
        signal.set(large.into_iter().rev().collect());
        assert_eq!(signal.get().len(), 10_000);
        let mut acc = 0u64;
        signal.with(|v| {
            for (i, item) in v.iter().enumerate() {
                acc += (i as u64) * u64::from(*item);
            }
        });
        // arbitrary sanity: acc non-zero for non-trivial input
        assert!(acc > 0);
    }

    // --- Cancellation (sync parts) ---

    /// A token flips to cancelled on `cancel`.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn cancellation_token_basic<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    /// Contexts are `Clone + Send + Sync`, per the trait contract.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub const fn reactive_context_is_clone_send_sync<C>(_cx: &C)
    where
        C: ReactiveContext + Clone + Send + Sync,
    {
        const fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
        assert_clone_send_sync::<C>();
    }

    /// `request_update` is always safe to call.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn request_update_does_not_panic<C: ReactiveContext>(cx: &C) {
        cx.request_update();
    }

    /// Several signals coexist independently.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn create_multiple_signals<C: ReactiveContext>(cx: &C) {
        let signal1 = cx.create_signal(1u32);
        let signal2 = cx.create_signal(2u32);
        assert_eq!(signal1.get(), 1);
        assert_eq!(signal2.get(), 2);
    }

    // --- use_global tests ---

    /// `use_global` on a missing global returns `None`, not a panic.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn use_global_does_not_panic<C: ReactiveContext>(cx: &C) {
        // Should not panic even if no global is registered.
        let _result: Option<<C as ReactiveContext>::Signal<String>> = cx.use_global();
    }

    // -----------------------------------------------------------------------------
    // Shared-globals semantics (unified across adapters)
    // -----------------------------------------------------------------------------

    /// Globals must be **shared**: mutations through one `use_global` signal
    /// are visible through another.
    ///
    /// This test pins the unified semantics introduced across all adapters
    /// (previously GPUI/Xilem/Floem/Dioxus returned independent copies while
    /// Leptos shared). All `use_global` consumers see the same cell.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn global_is_shared<C: ReactiveContext>(cx: &C) {
        cx.provide_global(10u32);
        let sig1 = cx.use_global::<u32>().expect("global must exist");
        let sig2 = cx.use_global::<u32>().expect("global must exist");
        sig1.set(55);
        assert_eq!(sig2.get(), 55, "use_global consumers must share state");
        assert_eq!(sig1.get(), 55);
    }

    // -----------------------------------------------------------------------------
    // Derived signals
    // -----------------------------------------------------------------------------

    /// Derived signals recompute after their source changes.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn derived_computes_and_tracks<C: ReactiveContext>(cx: &C) {
        let source = cx.create_signal(10u32);
        let doubled = quoin_core::derived(&source, |v| v * 2);
        assert_eq!(doubled.get(), 20);
        source.set(5);
        assert_eq!(doubled.get(), 10, "derived recomputes after source change");
    }

    /// Derived signals chain (`.map` of a derived).
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn derived_chains<C: ReactiveContext>(cx: &C) {
        let source = cx.create_signal(2i32);
        let chained = quoin_core::derived(&source, |v| v * 10).map(|v| v + 1);
        assert_eq!(chained.get(), 21);
        source.set(0);
        assert_eq!(chained.get(), 1);
    }

    // -----------------------------------------------------------------------------
    // Batched updates
    // -----------------------------------------------------------------------------

    /// `batch` runs the closure, returns its value, and collapses updates.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn batch_runs_and_returns<C: ReactiveContext>(cx: &C) {
        let a = cx.create_signal(0u32);
        let b = cx.create_signal(0u32);
        let result = cx.batch(|| {
            a.set(7);
            b.set(8);
            a.get() + b.get()
        });
        assert_eq!(result, 15);
        assert_eq!(a.get(), 7);
        assert_eq!(b.get(), 8);
    }

    // -----------------------------------------------------------------------------
    // SignalExt helpers
    // -----------------------------------------------------------------------------

    /// `set_eq` skips equal writes and applies changed ones.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn set_eq_skips_equal_writes<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal("x".to_string());
        assert!(
            !signal.set_eq("x".to_string()),
            "equal write must be skipped"
        );
        assert_eq!(signal.get(), "x");
        assert!(signal.set_eq("y".to_string()), "changed write must apply");
        assert_eq!(signal.get(), "y");
    }

    /// `replace` returns the previous value; `take` leaves `Default`.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn replace_and_take<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(1u32);
        assert_eq!(signal.replace(2), 1);
        assert_eq!(signal.get(), 2);
        assert_eq!(signal.take(), 2);
        assert_eq!(signal.get(), 0);
    }

    // -----------------------------------------------------------------------------
    // CancellationToken (sync parts)
    // -----------------------------------------------------------------------------

    /// Cancelling a parent token cancels its children.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn child_token_cascades<C: ReactiveContext>(_cx: &C) {
        let parent = CancellationToken::new();
        let child = parent.child_token();
        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled(), "parent cancel cascades to children");
    }

    /// The RAII cancel guard cancels when dropped.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn cancel_guard_on_drop<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        {
            let _guard = token.guard();
            assert!(!token.is_cancelled());
        }
        assert!(token.is_cancelled());
    }

    // -----------------------------------------------------------------------------
    // Executor tests (async: they await real task handles / timers)
    // -----------------------------------------------------------------------------

    /// A spawned task runs to completion and its side effects are visible.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn executor_spawn_success<C: ReactiveContext>(cx: &C)
    where
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<()>: std::future::IntoFuture,
    {
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = executor.spawn(async move {
            flag_clone.store(true, Ordering::SeqCst);
        });

        let _ = handle.await;
        assert!(flag.load(Ordering::SeqCst));
    }

    /// A task polling the token observes cancellation between steps.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn cancellation_token_cooperative<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: Clone,
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<()>: std::future::IntoFuture,
    {
        let token = CancellationToken::new();
        let clone = token.clone();

        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let executor_for_task = executor.clone();

        let handle = executor.spawn(async move {
            while !clone.is_cancelled() {
                executor_for_task.sleep(Duration::from_millis(1)).await;
            }
            flag_clone.store(true, Ordering::SeqCst);
        });

        executor.sleep(Duration::from_millis(50)).await;
        token.cancel();

        let _ = handle.await;
        assert!(flag.load(Ordering::SeqCst));
    }

    /// Verify `JoinHandle::abort` cancels the spawned task and that awaiting
    /// an aborted task yields `Err(Canceled)` instead of a result.
    /// `abort` cancels the task; awaiting resolves with `Err(Canceled)`.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn executor_abort_cancels_task<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: Clone,
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<()>:
            std::future::IntoFuture<Output = Result<(), futures::channel::oneshot::Canceled>>,
    {
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let task_executor = executor.clone();

        let handle = executor.spawn(async move {
            task_executor.sleep(Duration::from_secs(60)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Give the task a moment to start, then abort it.
        executor.sleep(Duration::from_millis(50)).await;
        handle.abort();

        let result = handle.await;
        assert!(
            result.is_err(),
            "awaiting an aborted task must resolve with Err(Canceled)"
        );
        assert!(
            !flag.load(Ordering::SeqCst),
            "aborted task body must not have run to completion"
        );
    }

    /// Verify `JoinHandle::detach` lets a task run to completion even though
    /// the handle is gone.
    /// `detach` lets the task finish even though the handle is gone.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn executor_detach_lets_task_finish<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: Clone,
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<()>:
            std::future::IntoFuture<Output = Result<(), futures::channel::oneshot::Canceled>>,
    {
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let task_executor = executor.clone();

        let handle = executor.spawn(async move {
            task_executor.sleep(Duration::from_millis(30)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        handle.detach();
        executor.sleep(Duration::from_millis(300)).await;
        assert!(
            flag.load(Ordering::SeqCst),
            "detached task must run to completion"
        );
    }

    /// `spawn_with_token` pairs a task with a cancellation token.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn spawn_with_token<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: Clone,
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<u32>:
            std::future::IntoFuture<Output = Result<u32, futures::channel::oneshot::Canceled>>,
    {
        use quoin_core::ExecutorExt;
        let executor = cx.executor();
        let task_executor = executor.clone();
        let (handle, token) = executor.spawn_with_token(move |token| async move {
            // Cooperative loop: exits as soon as the token is cancelled.
            while !token.is_cancelled() {
                task_executor.sleep(Duration::from_millis(1)).await;
            }
            1u32
        });
        let task_executor = cx.executor();
        task_executor.sleep(Duration::from_millis(20)).await;
        token.cancel();
        let result = handle.await;
        assert_eq!(result.ok(), Some(1));
    }

    /// `run_until_cancelled` returns `Some` when the future wins the race.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn run_until_cancelled_completes<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        let value = token.run_until_cancelled(async { 42u32 }).await;
        assert_eq!(value, Some(42));
    }

    /// `Executor::sleep` is part of the core contract: awaiting it must
    /// resolve (immediately or after the requested duration, depending on
    /// the executor's clock — gpui's test scheduler, for example, advances
    /// a *virtual* clock, so wall-clock assertions would be wrong here).
    ///
    /// # Panics
    /// Panics when the executor's sleep future never resolves.
    pub async fn executor_sleep_resolves<C: ReactiveContext>(cx: &C) {
        cx.executor().sleep(Duration::from_millis(1)).await;
    }

    /// A spawned task can await `Executor::sleep` internally (timers compose
    /// with spawned work — the pattern retry/backoff loops rely on).
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub async fn executor_spawned_task_sleeps<C: ReactiveContext>(cx: &C)
    where
        <<C as ReactiveContext>::Executor as Executor>::JoinHandle<()>: std::future::IntoFuture,
    {
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let executor_for_task = executor.clone();

        let handle = executor.spawn(async move {
            executor_for_task.sleep(Duration::from_millis(10)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        let _ = handle.await;
        assert!(
            flag.load(Ordering::SeqCst),
            "spawned task must observe its internal sleep"
        );
    }

    // -----------------------------------------------------------------------------
    // Local (non-Send) task execution
    // -----------------------------------------------------------------------------

    /// Local tasks are queued on the current thread (native) and must run when
    /// `run_local_tasks` drains the queue.
    ///
    /// On wasm they are platform-driven, so this test only asserts queue
    /// accounting consistency there. Detached local tasks run when the
    /// thread-local queue is drained.
    ///
    /// # Panics
    /// Panics when an assertion in this conformance check fails.
    pub fn local_task_lifecycle<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: quoin_core::LocalExecutor,
    {
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag2 = flag.clone();
        let handle = executor.spawn_local(async move {
            flag2.store(true, Ordering::SeqCst);
            7u32
        });
        handle.detach();
        let _ran = quoin_core::local::run_local_tasks();
        assert!(flag.load(Ordering::SeqCst), "detached local task must run");
    }
}

/// Verify `provide_global` / `use_global` round-trip (crate-level: also used
/// standalone by adapters that only need the global-state check).
///
/// # Panics
/// Panics when an assertion in this conformance check fails.
pub fn provide_and_use_global<C: ReactiveContext>(cx: &C) {
    cx.provide_global(99u32);
    let sig = cx
        .use_global::<u32>()
        .expect("global must exist after provide");
    assert_eq!(sig.get(), 99);
    sig.set(100);
    assert_eq!(sig.get(), 100);
}

// -----------------------------------------------------------------------------
// Test generation macros
// -----------------------------------------------------------------------------

/// Generates conformance tests for a framework adapter.
///
/// Two forms:
/// - `sync` for non-GPUI adapters that use `block_on` for the async subset.
/// - `gpui` for GPUI async tests.
///
/// Within each form, sync test functions run directly on the test thread and
/// async ones are wrapped in the harness's `block_on` (sync form) or emitted
/// as `async` test bodies (gpui form).
///
/// # Example
///
/// ```ignore
/// define_conformance_tests!(sync, MyTestHarness);
/// ```
#[macro_export]
macro_rules! define_conformance_tests {
    (sync, $cx_type:ty) => {
        use $crate::TestContextProvider;
        use $crate::provide_and_use_global;
        use $crate::tests::*;

        // Helpers defined inside the expansion so `$cx_type` is baked in
        // once, instead of being repeated in every generated test (DRY).
        macro_rules! __quoin_sync_test {
            ($test_name:ident, $fn_name:ident) => {
                #[test]
                fn $test_name() {
                    let cx = <$cx_type>::setup_context();
                    $fn_name(&cx);
                }
            };
        }
        macro_rules! __quoin_async_test {
            ($test_name:ident, $fn_name:ident) => {
                #[test]
                fn $test_name() {
                    let cx = <$cx_type>::setup_context();
                    <$cx_type>::block_on($fn_name(&cx));
                }
            };
        }

        // --- Sync (no executor round-trip required) ---
        __quoin_sync_test!(
            test_create_signal_initial_value,
            create_signal_initial_value
        );
        __quoin_sync_test!(test_signal_with_borrowing, signal_with_borrowing);
        __quoin_sync_test!(test_signal_clone_and_copy, signal_clone_and_copy);
        __quoin_sync_test!(test_signal_set_updates_value, signal_set_updates_value);
        __quoin_sync_test!(
            test_signal_update_modifies_value,
            signal_update_modifies_value
        );
        __quoin_sync_test!(
            test_signal_mutation_observable_via_with,
            signal_mutation_observable_via_with
        );
        __quoin_sync_test!(test_signal_clone_then_mutate, signal_clone_then_mutate);
        __quoin_sync_test!(test_signal_nested_with, signal_nested_with);
        __quoin_sync_test!(test_signal_large_value, signal_large_value);
        __quoin_sync_test!(test_cancellation_token_basic, cancellation_token_basic);
        __quoin_sync_test!(
            test_reactive_context_is_clone_send_sync,
            reactive_context_is_clone_send_sync
        );
        __quoin_sync_test!(
            test_request_update_does_not_panic,
            request_update_does_not_panic
        );
        __quoin_sync_test!(test_create_multiple_signals, create_multiple_signals);
        __quoin_sync_test!(test_use_global_does_not_panic, use_global_does_not_panic);
        __quoin_sync_test!(test_global_is_shared, global_is_shared);
        __quoin_sync_test!(test_provide_and_use_global, provide_and_use_global);
        __quoin_sync_test!(
            test_derived_computes_and_tracks,
            derived_computes_and_tracks
        );
        __quoin_sync_test!(test_derived_chains, derived_chains);
        __quoin_sync_test!(test_batch_runs_and_returns, batch_runs_and_returns);
        __quoin_sync_test!(test_set_eq_skips_equal_writes, set_eq_skips_equal_writes);
        __quoin_sync_test!(test_replace_and_take, replace_and_take);
        __quoin_sync_test!(test_child_token_cascades, child_token_cascades);
        __quoin_sync_test!(test_cancel_guard_on_drop, cancel_guard_on_drop);
        __quoin_sync_test!(test_local_task_lifecycle, local_task_lifecycle);

        // --- Async (executor-backed) ---
        __quoin_async_test!(test_executor_spawn_success, executor_spawn_success);
        __quoin_async_test!(
            test_cancellation_token_cooperative,
            cancellation_token_cooperative
        );
        __quoin_async_test!(
            test_executor_abort_cancels_task,
            executor_abort_cancels_task
        );
        __quoin_async_test!(
            test_executor_detach_lets_task_finish,
            executor_detach_lets_task_finish
        );
        __quoin_async_test!(test_spawn_with_token, spawn_with_token);
        __quoin_async_test!(
            test_run_until_cancelled_completes,
            run_until_cancelled_completes
        );
        __quoin_async_test!(test_executor_sleep_resolves, executor_sleep_resolves);
        __quoin_async_test!(
            test_executor_spawned_task_sleeps,
            executor_spawned_task_sleeps
        );
    };

    (gpui, $cx_type:ty) => {
        use $crate::provide_and_use_global;
        use $crate::tests::*;
        // Expect `gpui::TestAppContext` to be in scope.

        macro_rules! __quoin_gpui_sync_test {
            ($test_name:ident, $fn_name:ident) => {
                #[gpui::test]
                fn $test_name(cx: &mut TestAppContext) {
                    let harness = <$cx_type>::new(cx);
                    $fn_name(&harness);
                }
            };
        }
        macro_rules! __quoin_gpui_async_test {
            ($test_name:ident, $fn_name:ident) => {
                // The conformance future is driven by `block_test` on the test
                // scheduler — exactly how `#[gpui::test]` async bodies run
                // (including virtual-timer advancement). Keeping the test
                // function synchronous avoids a `!Send` future: `TestAppContext`
                // is `Rc`-backed and deliberately `!Send`, so an `async fn`
                // parameter would trip `clippy::future_not_send` even though
                // the harness is single-threaded by design.
                #[gpui::test]
                fn $test_name(cx: &mut TestAppContext) {
                    let harness = <$cx_type>::new(cx);
                    let foreground = cx.update(|cx| cx.foreground_executor().clone());
                    foreground.block_test(async move {
                        $fn_name(&harness).await;
                    });
                }
            };
        }

        // --- Sync (no await needed) ---
        __quoin_gpui_sync_test!(
            test_create_signal_initial_value,
            create_signal_initial_value
        );
        __quoin_gpui_sync_test!(test_signal_with_borrowing, signal_with_borrowing);
        __quoin_gpui_sync_test!(test_signal_clone_and_copy, signal_clone_and_copy);
        __quoin_gpui_sync_test!(test_signal_set_updates_value, signal_set_updates_value);
        __quoin_gpui_sync_test!(
            test_signal_update_modifies_value,
            signal_update_modifies_value
        );
        __quoin_gpui_sync_test!(
            test_signal_mutation_observable_via_with,
            signal_mutation_observable_via_with
        );
        __quoin_gpui_sync_test!(test_signal_clone_then_mutate, signal_clone_then_mutate);
        __quoin_gpui_sync_test!(test_signal_nested_with, signal_nested_with);
        __quoin_gpui_sync_test!(test_signal_large_value, signal_large_value);
        __quoin_gpui_sync_test!(test_cancellation_token_basic, cancellation_token_basic);
        __quoin_gpui_sync_test!(
            test_reactive_context_is_clone_send_sync,
            reactive_context_is_clone_send_sync
        );
        __quoin_gpui_sync_test!(
            test_request_update_does_not_panic,
            request_update_does_not_panic
        );
        __quoin_gpui_sync_test!(test_create_multiple_signals, create_multiple_signals);
        __quoin_gpui_sync_test!(test_use_global_does_not_panic, use_global_does_not_panic);
        __quoin_gpui_sync_test!(test_global_is_shared, global_is_shared);
        __quoin_gpui_sync_test!(test_provide_and_use_global, provide_and_use_global);
        __quoin_gpui_sync_test!(
            test_derived_computes_and_tracks,
            derived_computes_and_tracks
        );
        __quoin_gpui_sync_test!(test_derived_chains, derived_chains);
        __quoin_gpui_sync_test!(test_batch_runs_and_returns, batch_runs_and_returns);
        __quoin_gpui_sync_test!(test_set_eq_skips_equal_writes, set_eq_skips_equal_writes);
        __quoin_gpui_sync_test!(test_replace_and_take, replace_and_take);
        __quoin_gpui_sync_test!(test_child_token_cascades, child_token_cascades);
        __quoin_gpui_sync_test!(test_cancel_guard_on_drop, cancel_guard_on_drop);

        // --- Async (executor-backed) ---
        __quoin_gpui_async_test!(test_executor_spawn_success, executor_spawn_success);
        __quoin_gpui_async_test!(
            test_cancellation_token_cooperative,
            cancellation_token_cooperative
        );
        __quoin_gpui_async_test!(
            test_executor_abort_cancels_task,
            executor_abort_cancels_task
        );
        __quoin_gpui_async_test!(
            test_executor_detach_lets_task_finish,
            executor_detach_lets_task_finish
        );
        __quoin_gpui_async_test!(test_spawn_with_token, spawn_with_token);
        __quoin_gpui_async_test!(
            test_run_until_cancelled_completes,
            run_until_cancelled_completes
        );
        __quoin_gpui_async_test!(test_executor_sleep_resolves, executor_sleep_resolves);
        __quoin_gpui_async_test!(
            test_executor_spawned_task_sleeps,
            executor_spawned_task_sleeps
        );
    };
}
