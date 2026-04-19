//! GPUI conformance tests – manually implemented using `#[gpui::test]`

use gpui::TestAppContext;
use quoin::{CancellationToken, Executor, ReactiveContext, Signal};
use quoin_gpui::GpuiContext;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// -----------------------------------------------------------------------------
// Test Harness
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct TestHarness {
    context: GpuiContext,
}

impl TestHarness {
    fn new(cx: &mut TestAppContext) -> Self {
        let foreground = cx.update(|cx| cx.foreground_executor().clone());
        let background = cx.executor().clone();
        let context = GpuiContext::for_test(foreground, background);
        Self { context }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <GpuiContext as ReactiveContext>::Signal<T>;
    type Executor = <GpuiContext as ReactiveContext>::Executor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.context.create_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        self.context.executor()
    }

    fn request_update(&self) {
        self.context.request_update()
    }
}

// -----------------------------------------------------------------------------
// Macro to generate GPUI test functions
// -----------------------------------------------------------------------------

macro_rules! gpui_conformance_test {
    ($name:ident, $body:expr) => {
        #[gpui::test]
        async fn $name(cx: &mut TestAppContext) {
            let harness = TestHarness::new(cx);
            let fut = ($body)(harness);
            fut.await;
        }
    };
}

// -----------------------------------------------------------------------------
// Signal Tests
// -----------------------------------------------------------------------------

gpui_conformance_test!(
    test_create_signal_initial_value,
    (|harness: TestHarness| async move {
        let signal = harness.create_signal(42u32);
        assert_eq!(signal.get(), 42);
    })
);

gpui_conformance_test!(
    test_signal_with_borrowing,
    (|harness: TestHarness| async move {
        let signal = harness.create_signal("hello".to_string());
        signal.with(|value| {
            assert_eq!(value, "hello");
        });
    })
);

gpui_conformance_test!(
    test_signal_clone_and_copy,
    (|harness: TestHarness| async move {
        let signal = harness.create_signal(100u32);
        let copy = signal;
        assert_eq!(copy.get(), 100);
    })
);

// -----------------------------------------------------------------------------
// Executor Tests
// -----------------------------------------------------------------------------

gpui_conformance_test!(
    test_executor_spawn_success,
    (|harness: TestHarness| async move {
        let executor = harness.executor();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = executor.spawn(async move {
            flag_clone.store(true, Ordering::SeqCst);
        });

        let _ = handle.await;
        assert!(flag.load(Ordering::SeqCst));
    })
);

gpui_conformance_test!(
    test_cancellation_token_basic,
    (|_harness: TestHarness| async move {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    })
);

gpui_conformance_test!(
    test_cancellation_token_cooperative,
    (|harness: TestHarness| async move {
        let token = CancellationToken::new();
        let clone = token.clone();

        let executor = harness.executor();

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        // Clone harness before moving into the spawned task
        let harness_for_task = harness.clone();

        let handle = executor.spawn(async move {
            while !clone.is_cancelled() {
                // Use executor's sleep (test‑scheduler friendly)
                harness_for_task
                    .executor()
                    .sleep(Duration::from_millis(1))
                    .await;
            }
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Cancel after a short delay.
        harness.executor().sleep(Duration::from_millis(50)).await;
        token.cancel();

        let _ = handle.await;
        assert!(flag.load(Ordering::SeqCst));
    })
);

// -----------------------------------------------------------------------------
// Context Tests
// -----------------------------------------------------------------------------

gpui_conformance_test!(
    test_reactive_context_is_clone_send_sync,
    (|_harness: TestHarness| async move {
        fn assert_clone_send_sync<T: Clone + Send + Sync>() {}
        assert_clone_send_sync::<TestHarness>();
    })
);

gpui_conformance_test!(
    test_request_update_does_not_panic,
    (|harness: TestHarness| async move {
        harness.request_update();
    })
);

gpui_conformance_test!(
    test_create_multiple_signals,
    (|harness: TestHarness| async move {
        let signal1 = harness.create_signal(1u32);
        let signal2 = harness.create_signal(2u32);
        assert_eq!(signal1.get(), 1);
        assert_eq!(signal2.get(), 2);
    })
);
