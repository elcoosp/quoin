use leptos::prelude::*;
use quoin_core::{Executor, ReactiveContext};
use quoin_conformance::{SleepExt, define_conformance_tests};
use quoin_leptos::{LeptosContext, LeptosExecutor};
use std::future::Future;
use std::time::Duration;

#[derive(Clone)]
struct TestExecutor(LeptosExecutor);

impl SleepExt for TestExecutor {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        futures_timer::Delay::new(duration)
    }
}

impl Executor for TestExecutor {
    type JoinHandle<T: Send + 'static> = <LeptosExecutor as Executor>::JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

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
        Self {
            context: LeptosContext::new(),
            _owner: owner,
        }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <LeptosContext as ReactiveContext>::Signal<T>;
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

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.context.use_global()
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

define_conformance_tests!(sync, TestHarness);
