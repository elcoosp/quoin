//! Conformance test suite for `quoin` framework adapters.
//!
//! This crate provides a trait that defines a set of tests any `ReactiveContext`
//! implementation must pass. Adapter crates use the `#[test_impl]` macro from
//! `tested_trait` to run these tests against their concrete context type.
//!
//! # Usage
//!
//! ```rust,ignore
//! use quoin::ReactiveContext;
//! use quoin_conformance::ReactiveContextConformance;
//! use tested_trait::test_impl;
//!
//! #[test_impl]
//! impl ReactiveContextConformance for MyAdapterContext {
//!     fn setup_context() -> Self {
//!         // Create a fresh context for testing
//!         MyAdapterContext::new_for_test()
//!     }
//! }
//! ```

use futures::Future;
use quoin::{CancellationToken, Executor, ReactiveContext, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tested_trait::tested_trait;

/// A conformance test suite for `ReactiveContext` implementations.
///
/// This trait contains the tests that verify an adapter behaves according to
/// the `quoin` specification. It should be implemented for the concrete
/// `ReactiveContext` type provided by the adapter.
///
/// The only required method is `setup_context()`, which should return a fresh
/// context instance suitable for testing.
#[tested_trait]
pub trait ReactiveContextConformance: ReactiveContext {
    /// Creates a fresh context instance for testing.
    ///
    /// This method is called before each test. The context should be in a
    /// clean state with no pre-existing signals or side effects.
    fn setup_context() -> Self;

    /// Helper to block on a future in tests.
    ///
    /// The default implementation uses `futures::executor::block_on`,
    /// but adapters for environments without a standard executor (e.g., WASM)
    /// may override this method.
    fn block_on<F: Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }

    // ------------------------------------------------------------------------
    // Signal Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_create_signal_initial_value() {
        let cx = Self::setup_context();
        let signal = cx.create_signal(42u32);
        assert_eq!(signal.get(), 42);
    }

    #[test]
    fn test_signal_with_borrowing() {
        let cx = Self::setup_context();
        let signal = cx.create_signal("hello".to_string());
        signal.with(|value| {
            assert_eq!(value, "hello");
        });
    }

    #[test]
    fn test_signal_clone_and_copy() {
        let cx = Self::setup_context();
        let signal = cx.create_signal(100u32);
        let copy = signal;
        assert_eq!(copy.get(), 100);
    }

    // ------------------------------------------------------------------------
    // Executor Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_executor_spawn_success() {
        let cx = Self::setup_context();
        let executor = cx.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let _handle = executor.spawn(async move {
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Busy-wait for the task to execute (max 5 seconds)
        let start = std::time::Instant::now();
        while !flag.load(Ordering::SeqCst) {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("task did not execute within 5 seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn test_cancellation_token_basic() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_cooperative() {
        let token = CancellationToken::new();
        let clone = token.clone();

        let cx = Self::setup_context();
        let executor = cx.executor();

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        // Spawn a task that checks cancellation
        let _handle = executor.spawn(async move {
            while !clone.is_cancelled() {
                // Busy loop, but yields to executor
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Cancel after a short delay
        std::thread::sleep(std::time::Duration::from_millis(50));
        token.cancel();

        // Wait for flag to be set (max 5 seconds)
        let start = std::time::Instant::now();
        while !flag.load(Ordering::SeqCst) {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                panic!("task did not respond to cancellation within 5 seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // ------------------------------------------------------------------------
    // Context Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_reactive_context_is_clone_send_sync() {
        fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
        assert_clone_send_sync::<Self>();
    }

    #[test]
    fn test_request_update_does_not_panic() {
        let cx = Self::setup_context();
        cx.request_update();
        // No assertion; just ensuring it doesn't panic.
    }

    #[test]
    fn test_create_multiple_signals() {
        let cx = Self::setup_context();
        let signal1 = cx.create_signal(1u32);
        let signal2 = cx.create_signal(2u32);
        assert_eq!(signal1.get(), 1);
        assert_eq!(signal2.get(), 2);
    }
}
