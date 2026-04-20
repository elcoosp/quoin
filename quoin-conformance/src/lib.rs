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
//! use quoin::ReactiveContext;
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
//! use quoin::ReactiveContext;
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

use quoin::{CancellationToken, Executor, ReactiveContext, Signal};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

    // --- New mutation tests ---

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

    // --- Existing executor and cancellation tests ---

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
}

// -----------------------------------------------------------------------------
// Test generation macros
// -----------------------------------------------------------------------------

#[macro_export]
macro_rules! define_conformance_tests {
    (sync, $cx_type:ty) => {
        use $crate::tests::*;
        use $crate::TestContextProvider;

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
    };

    (gpui, $cx_type:ty) => {
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
    };
}
