//! Conformance test suite for `quoin` framework adapters.
//!
//! This crate provides a set of reusable test functions that verify an adapter's
//! `ReactiveContext` implementation meets the specification. Adapter crates use
//! the `define_conformance_tests!` macro to generate the actual test functions
//! with the appropriate test runner.
//!
//! # Usage for non‑GPUI adapters
//!
//! ```rust,ignore
//! use quoin_core::ReactiveContext;
//! use quoin_conformance::{define_conformance_tests, TestContextProvider};
//!
//! struct MyTestHarness { ... }
//! impl ReactiveContext for MyTestHarness { ... }
//! impl TestContextProvider for MyTestHarness {
//!     fn setup_context() -> Self { ... }
//!     fn block_on<F: Future>(future: F) -> F::Output { ... }
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
//! use quoin_conformance::{define_conformance_tests, SleepExt};
//!
//! struct TestHarness { context: GpuiContext }
//! impl TestHarness {
//!     fn new(cx: &mut TestAppContext) -> Self { ... }
//! }
//! impl ReactiveContext for TestHarness { ... }
//! impl SleepExt for <GpuiContext as ReactiveContext>::Executor {
//!     async fn sleep(&self, duration: Duration) { ... }
//! }
//!
//! define_conformance_tests!(gpui, TestHarness);
//! ```

use quoin_core::prelude::*;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

// -----------------------------------------------------------------------------
// Helper traits
// -----------------------------------------------------------------------------

pub trait TestContextProvider: ReactiveContext + Sized {
    fn setup_context() -> Self;
    fn block_on<F: Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }
}

pub trait SleepExt {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

// -----------------------------------------------------------------------------
// Core async test functions
// -----------------------------------------------------------------------------

pub mod tests {
    use super::*;

    pub async fn create_signal_initial_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        assert_eq!(signal.get(), 42);
    }

    pub async fn signal_with_borrowing<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal("hello".to_string());
        signal.with(|value| {
            assert_eq!(value, "hello");
        });
    }

    pub async fn signal_clone_and_copy<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(100u32);
        let copy = signal;
        assert_eq!(copy.get(), 100);
    }

    // --- Mutation tests ---

    pub async fn signal_set_updates_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(0u32);
        signal.set(42);
        assert_eq!(signal.get(), 42);
    }

    pub async fn signal_update_modifies_value<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(0u32);
        signal.update(|v| *v += 5);
        signal.update(|v| *v *= 2);
        assert_eq!(signal.get(), 10);
    }

    pub async fn signal_mutation_observable_via_with<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(String::from("hello"));
        signal.update(|s| s.push_str(" world"));
        signal.with(|s| assert_eq!(s, "hello world"));
    }

    // --- Edge‑case tests ---

    /// Verify that a clone of a signal shares the same state
    pub async fn signal_clone_then_mutate<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        let clone = signal.clone();
        clone.set(99);
        assert_eq!(signal.get(), 99);
    }

    /// Ensure nested `with` calls do not deadlock or panic
    pub async fn signal_nested_with<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(42u32);
        signal.with(|outer| {
            signal.with(|inner| {
                assert_eq!(*outer, *inner);
            });
        });
    }

    /// Stress test with a large value (10k elements)
    pub async fn signal_large_value<C: ReactiveContext>(cx: &C) {
        let large: Vec<u32> = (0..10_000).collect();
        let signal = cx.create_signal(large.clone());
        assert_eq!(signal.get(), large);
        signal.set(large.clone().into_iter().rev().collect());
        assert_eq!(signal.get().len(), 10_000);
        let mut acc = 0u64;
        signal.with(|v| {
            for (i, item) in v.iter().enumerate() {
                acc += (i as u64) * (*item as u64);
            }
        });
        // arbitrary sanity: acc non‑zero for non‑trivial input
        assert!(acc > 0);
    }

    // --- Executor and cancellation tests ---

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

    pub async fn cancellation_token_basic<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    pub async fn cancellation_token_cooperative<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: SleepExt + Clone,
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
    pub async fn executor_abort_cancels_task<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: SleepExt + Clone,
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
    pub async fn executor_detach_lets_task_finish<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: SleepExt + Clone,
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

    pub async fn reactive_context_is_clone_send_sync<C>(_cx: &C)
    where
        C: ReactiveContext + Clone + Send + Sync,
    {
        fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
        assert_clone_send_sync::<C>();
    }

    pub async fn request_update_does_not_panic<C: ReactiveContext>(cx: &C) {
        cx.request_update();
    }

    pub async fn create_multiple_signals<C: ReactiveContext>(cx: &C) {
        let signal1 = cx.create_signal(1u32);
        let signal2 = cx.create_signal(2u32);
        assert_eq!(signal1.get(), 1);
        assert_eq!(signal2.get(), 2);
    }

    // --- use_global test ---

    pub async fn use_global_does_not_panic<C: ReactiveContext>(cx: &C) {
        // Should not panic even if no global is registered.
        let _result: Option<<C as ReactiveContext>::Signal<String>> = cx.use_global();
    }
    // -----------------------------------------------------------------------------
    // Shared-globals semantics (unified across adapters)
    // -----------------------------------------------------------------------------

    /// Globals must be **shared**: mutations through one `use_global` signal are
    /// visible through another. This test pins the unified semantics introduced
    /// across all adapters (previously GPUI/Xilem/Floem/Dioxus returned
    /// independent copies while Leptos shared).
    pub async fn global_is_shared<C: ReactiveContext>(cx: &C) {
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

    pub async fn derived_computes_and_tracks<C: ReactiveContext>(cx: &C) {
        let source = cx.create_signal(10u32);
        let doubled = quoin_core::derived(&source, |v| v * 2);
        assert_eq!(doubled.get(), 20);
        source.set(5);
        assert_eq!(doubled.get(), 10, "derived recomputes after source change");
    }

    pub async fn derived_chains<C: ReactiveContext>(cx: &C) {
        let source = cx.create_signal(2i32);
        let chained = quoin_core::derived(&source, |v| v * 10).map(|v| v + 1);
        assert_eq!(chained.get(), 21);
        source.set(0);
        assert_eq!(chained.get(), 1);
    }

    // -----------------------------------------------------------------------------
    // Batched updates
    // -----------------------------------------------------------------------------

    pub async fn batch_runs_and_returns<C: ReactiveContext>(cx: &C) {
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

    pub async fn set_eq_skips_equal_writes<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal("x".to_string());
        assert!(
            !signal.set_eq("x".to_string()),
            "equal write must be skipped"
        );
        assert_eq!(signal.get(), "x");
        assert!(signal.set_eq("y".to_string()), "changed write must apply");
        assert_eq!(signal.get(), "y");
    }

    pub async fn replace_and_take<C: ReactiveContext>(cx: &C) {
        let signal = cx.create_signal(1u32);
        assert_eq!(signal.replace(2), 1);
        assert_eq!(signal.get(), 2);
        assert_eq!(signal.take(), 2);
        assert_eq!(signal.get(), 0);
    }

    // -----------------------------------------------------------------------------
    // ExecutorExt / spawn_with_token
    // -----------------------------------------------------------------------------

    pub async fn spawn_with_token<C>(cx: &C)
    where
        C: ReactiveContext,
        C::Executor: SleepExt + Clone,
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

    // -----------------------------------------------------------------------------
    // CancellationToken upgrades
    // -----------------------------------------------------------------------------

    pub async fn child_token_cascades<C: ReactiveContext>(_cx: &C) {
        let parent = CancellationToken::new();
        let child = parent.child_token();
        assert!(!child.is_cancelled());
        parent.cancel();
        assert!(child.is_cancelled(), "parent cancel cascades to children");
    }

    pub async fn cancel_guard_on_drop<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        {
            let _guard = token.guard();
            assert!(!token.is_cancelled());
        }
        assert!(token.is_cancelled());
    }

    pub async fn run_until_cancelled_completes<C: ReactiveContext>(_cx: &C) {
        let token = CancellationToken::new();
        let value = token.run_until_cancelled(async { 42u32 }).await;
        assert_eq!(value, Some(42));
    }

    // -----------------------------------------------------------------------------
    // Local (non-Send) task execution
    // -----------------------------------------------------------------------------

    /// Local tasks are queued on the current thread (native) and must run when
    /// `run_local_tasks` drains the queue. On wasm they are platform-driven, so
    /// this test only asserts queue accounting consistency there.
    pub async fn local_task_lifecycle<C>(cx: &C)
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

/// Verify provide_global / use_global round-trip.
pub async fn provide_and_use_global<C: ReactiveContext>(cx: &C) {
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

#[macro_export]
/// Generates conformance tests for a framework adapter.
///
/// Two forms:
/// - `sync` for non‑GPUI adapters that use `block_on`.
/// - `gpui` for GPUI async tests.
///
/// # Example
///
/// ```ignore
/// define_conformance_tests!(sync, MyTestHarness);
/// ```
macro_rules! define_conformance_tests {
    (sync, $cx_type:ty) => {
        use $crate::TestContextProvider;
        use $crate::provide_and_use_global;
        use $crate::tests::*;

        #[test]
        fn test_create_signal_initial_value() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(create_signal_initial_value(&cx));
        }

        #[test]
        fn test_signal_with_borrowing() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_with_borrowing(&cx));
        }

        #[test]
        fn test_signal_clone_and_copy() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_clone_and_copy(&cx));
        }

        #[test]
        fn test_signal_set_updates_value() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_set_updates_value(&cx));
        }

        #[test]
        fn test_signal_update_modifies_value() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_update_modifies_value(&cx));
        }

        #[test]
        fn test_signal_mutation_observable_via_with() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_mutation_observable_via_with(&cx));
        }

        #[test]
        fn test_signal_clone_then_mutate() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_clone_then_mutate(&cx));
        }

        #[test]
        fn test_signal_nested_with() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_nested_with(&cx));
        }

        #[test]
        fn test_signal_large_value() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(signal_large_value(&cx));
        }

        #[test]
        fn test_executor_spawn_success() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(executor_spawn_success(&cx));
        }

        #[test]
        fn test_cancellation_token_basic() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(cancellation_token_basic(&cx));
        }

        #[test]
        fn test_cancellation_token_cooperative() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(cancellation_token_cooperative(&cx));
        }

        #[test]
        fn test_executor_abort_cancels_task() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(executor_abort_cancels_task(&cx));
        }

        #[test]
        fn test_executor_detach_lets_task_finish() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(executor_detach_lets_task_finish(&cx));
        }

        #[test]
        fn test_reactive_context_is_clone_send_sync() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(reactive_context_is_clone_send_sync(&cx));
        }

        #[test]
        fn test_request_update_does_not_panic() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(request_update_does_not_panic(&cx));
        }

        #[test]
        fn test_create_multiple_signals() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(create_multiple_signals(&cx));
        }

        #[test]
        fn test_use_global_does_not_panic() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(use_global_does_not_panic(&cx));
        }

        #[test]
        fn test_provide_and_use_global() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(provide_and_use_global(&cx));
        }

        #[test]
        fn test_global_is_shared() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(global_is_shared(&cx));
        }

        #[test]
        fn test_derived_computes_and_tracks() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(derived_computes_and_tracks(&cx));
        }

        #[test]
        fn test_derived_chains() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(derived_chains(&cx));
        }

        #[test]
        fn test_batch_runs_and_returns() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(batch_runs_and_returns(&cx));
        }

        #[test]
        fn test_set_eq_skips_equal_writes() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(set_eq_skips_equal_writes(&cx));
        }

        #[test]
        fn test_replace_and_take() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(replace_and_take(&cx));
        }

        #[test]
        fn test_spawn_with_token() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(spawn_with_token(&cx));
        }

        #[test]
        fn test_child_token_cascades() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(child_token_cascades(&cx));
        }

        #[test]
        fn test_cancel_guard_on_drop() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(cancel_guard_on_drop(&cx));
        }

        #[test]
        fn test_run_until_cancelled_completes() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(run_until_cancelled_completes(&cx));
        }

        #[test]
        fn test_local_task_lifecycle() {
            let cx = <$cx_type>::setup_context();
            <$cx_type>::block_on(local_task_lifecycle(&cx));
        }
    };

    (gpui, $cx_type:ty) => {
        use $crate::provide_and_use_global;
        use $crate::tests::*;
        // Expect `gpui::TestAppContext` to be in scope.

        #[gpui::test]
        async fn test_create_signal_initial_value(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            create_signal_initial_value(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_with_borrowing(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_with_borrowing(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_clone_and_copy(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_clone_and_copy(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_set_updates_value(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_set_updates_value(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_update_modifies_value(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_update_modifies_value(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_mutation_observable_via_with(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_mutation_observable_via_with(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_clone_then_mutate(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_clone_then_mutate(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_nested_with(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_nested_with(&harness).await;
        }

        #[gpui::test]
        async fn test_signal_large_value(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            signal_large_value(&harness).await;
        }

        #[gpui::test]
        async fn test_executor_spawn_success(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            executor_spawn_success(&harness).await;
        }

        #[gpui::test]
        async fn test_cancellation_token_basic(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            cancellation_token_basic(&harness).await;
        }

        #[gpui::test]
        async fn test_cancellation_token_cooperative(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            cancellation_token_cooperative(&harness).await;
        }

        #[gpui::test]
        async fn test_executor_abort_cancels_task(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            executor_abort_cancels_task(&harness).await;
        }

        #[gpui::test]
        async fn test_executor_detach_lets_task_finish(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            executor_detach_lets_task_finish(&harness).await;
        }

        #[gpui::test]
        async fn test_reactive_context_is_clone_send_sync(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            reactive_context_is_clone_send_sync(&harness).await;
        }

        #[gpui::test]
        async fn test_request_update_does_not_panic(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            request_update_does_not_panic(&harness).await;
        }

        #[gpui::test]
        async fn test_create_multiple_signals(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            create_multiple_signals(&harness).await;
        }

        #[gpui::test]
        async fn test_use_global_does_not_panic(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            use_global_does_not_panic(&harness).await;
        }

        #[gpui::test]
        async fn test_provide_and_use_global(cx: &mut TestAppContext) {
            let harness = <$cx_type>::new(cx);
            provide_and_use_global(&harness).await;
        }
    };
}
