use leptos::prelude::*;
use quoin::ReactiveContext;
use quoin_conformance::ReactiveContextConformance;
use quoin_leptos::LeptosContext;
use tested_trait::test_impl;

#[derive(Clone)]
struct TestHarness {
    context: LeptosContext,
}

impl TestHarness {
    fn new() -> Self {
        // In Leptos 0.8, we can create a runtime with `create_runtime()`
        let runtime = create_runtime();
        let context = LeptosContext::new();
        Self { context }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <LeptosContext as ReactiveContext>::Signal<T>;
    type Executor = <LeptosContext as ReactiveContext>::Executor;

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
