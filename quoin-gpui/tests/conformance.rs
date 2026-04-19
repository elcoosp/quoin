use gpui::TestAppContext;
use quoin::{Executor, JoinHandle, ReactiveContext};
use quoin_conformance::{define_conformance_tests, SleepExt};
use quoin_gpui::{GpuiContext, GpuiExecutor};
use std::future::Future;
use std::time::Duration;

// -----------------------------------------------------------------------------
// Newtype wrapper for GpuiExecutor to implement SleepExt without orphan rule
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct TestExecutor(GpuiExecutor);

impl SleepExt for TestExecutor {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        self.0.sleep(duration)
    }
}

// Delegate all Executor methods to the inner GpuiExecutor.
impl Executor for TestExecutor {
    type JoinHandle<T: Send + 'static> = <GpuiExecutor as Executor>::JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

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
        Self {
            context: GpuiContext::for_test(foreground, background),
        }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <GpuiContext as ReactiveContext>::Signal<T>;
    type Executor = TestExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.context.create_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        TestExecutor(self.context.executor())
    }

    fn request_update(&self) {
        self.context.request_update()
    }
}

// -----------------------------------------------------------------------------
// Generate all conformance tests
// -----------------------------------------------------------------------------

define_conformance_tests!(gpui, TestHarness);
