use dioxus_core::{Element, ScopeId, VNode, VirtualDom};
use quoin::{Executor, ReactiveContext};
use quoin_conformance::{define_conformance_tests, SleepExt};
use quoin_dioxus::{DioxusContext, DioxusExecutor};
use std::future::Future;
use std::time::Duration;

fn app() -> Element {
    VNode::empty()
}

// -----------------------------------------------------------------------------
// Newtype wrapper to implement SleepExt without orphan rule
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct TestExecutor(DioxusExecutor);

impl SleepExt for TestExecutor {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        futures_timer::Delay::new(duration)
    }
}

impl Executor for TestExecutor {
    type JoinHandle<T: Send + 'static> = <DioxusExecutor as Executor>::JoinHandle<T>;

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

struct TestHarness {
    context: DioxusContext,
    vdom: Box<VirtualDom>,
}

// SAFETY: The test harness is only used on the main thread in conformance tests.
unsafe impl Send for TestHarness {}
unsafe impl Sync for TestHarness {}

impl Clone for TestHarness {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl TestHarness {
    fn new() -> Self {
        let mut vdom = VirtualDom::new(app);
        vdom.rebuild_in_place();
        Self {
            context: DioxusContext::new(),
            vdom: Box::new(vdom),
        }
    }

    fn with_vdom<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.vdom.in_scope(ScopeId::ROOT, f)
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <DioxusContext as ReactiveContext>::Signal<T>;
    type Executor = TestExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.with_vdom(|| self.context.create_signal(initial))
    }

    fn executor(&self) -> Self::Executor {
        self.with_vdom(|| TestExecutor(self.context.executor()))
    }

    fn request_update(&self) {
        self.with_vdom(|| self.context.request_update())
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
