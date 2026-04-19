use leptos::prelude::*;
use quoin::ReactiveContext;
use quoin_conformance::{define_conformance_tests, TestContextProvider};
use quoin_leptos::LeptosContext;
use std::future::Future;

struct TestHarness {
    context: LeptosContext,
    _owner: Owner,
}

impl Clone for TestHarness {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            _owner: self._owner.clone(),
        }
    }
}

impl TestHarness {
    fn new() -> Self {
        let owner = Owner::new();
        owner.set();

        let context = LeptosContext::new();
        Self {
            context,
            _owner: owner,
        }
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

impl TestContextProvider for TestHarness {
    fn setup_context() -> Self {
        Self::new()
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }
}

// Generate all conformance tests synchronously.
define_conformance_tests!(sync, TestHarness);
