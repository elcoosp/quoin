//! Conformance tests for the quoin-floem adapter.
use quoin_conformance::define_conformance_tests;
use quoin_core::{Executor, ReactiveContext};
use quoin_floem::{FloemContext, FloemExecutor};
use std::future::Future;
use std::time::Duration;

#[derive(Clone)]
struct TestExecutor(FloemExecutor);

impl Executor for TestExecutor {
    type JoinHandle<T: Send + 'static> = <FloemExecutor as Executor>::JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

struct TestHarness {
    context: FloemContext,
}

impl Clone for TestHarness {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl TestHarness {
    const fn new() -> Self {
        Self {
            context: FloemContext::new(),
        }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <FloemContext as ReactiveContext>::Signal<T>;
    type Executor = TestExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.context.create_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        TestExecutor(self.context.executor())
    }

    fn request_update(&self) {
        self.context.request_update();
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        self.context.provide_global(value);
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.context.use_global()
    }
}

impl quoin_core::LocalExecutor for TestExecutor {
    type JoinHandle<T: 'static> = quoin_core::LocalTask<T>;

    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: std::future::Future + 'static,
        F::Output: 'static,
    {
        quoin_core::spawn_local_task(future)
    }
}

impl quoin_conformance::TestContextProvider for TestHarness {
    fn setup_context() -> Self {
        Self::new()
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        futures::executor::block_on(future)
    }
}

define_conformance_tests!(sync, TestHarness);
