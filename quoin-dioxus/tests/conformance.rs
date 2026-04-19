use dioxus::prelude::*;
use quoin::ReactiveContext;
use quoin_conformance::ReactiveContextConformance;
use quoin_dioxus::DioxusContext;
use tested_trait::test_impl;

#[derive(Clone)]
struct TestHarness {
    context: DioxusContext,
}

impl TestHarness {
    fn new() -> Self {
        dioxus::prelude::launch(|| rsx! { div {} });
        let context = DioxusContext::new();
        Self { context }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <DioxusContext as ReactiveContext>::Signal<T>;
    type Executor = <DioxusContext as ReactiveContext>::Executor;

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
