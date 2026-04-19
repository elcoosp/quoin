// quoin-gpui/tests/conformance.rs
use gpui::{TestApp, WindowOptions};
use quoin::ReactiveContext;
use quoin_conformance::ReactiveContextConformance;
use quoin_gpui::GpuiContext;
use tested_trait::test_impl;

#[derive(Clone)]
struct TestHarness {
    context: GpuiContext,
}

impl TestHarness {
    fn new() -> Self {
        // Create a test application – this does NOT start an event loop.
        let app = TestApp::new();
        // Extract the foreground executor from the test app.
        let foreground = app.foreground_executor().clone();
        // Build GpuiContext directly from the executor.
        let context = GpuiContext::from_executor(foreground);
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

#[test_impl]
impl ReactiveContextConformance for TestHarness {
    fn setup_context() -> Self {
        Self::new()
    }
}
