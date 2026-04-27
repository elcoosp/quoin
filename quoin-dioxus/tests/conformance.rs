use dioxus_core::{Element, ScopeId, VNode, VirtualDom};
use quoin_conformance::{SleepExt, define_conformance_tests};
use quoin_core::{Executor, ReactiveContext};
use quoin_dioxus::{DioxusContext, DioxusExecutor};
use std::future::Future;
use std::time::Duration;

fn app() -> Element {
    VNode::empty()
}

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

thread_local! {
    static TEST_VDOM: std::cell::RefCell<Option<Box<VirtualDom>>> = std::cell::RefCell::new(None);
}

struct TestHarness {
    context: DioxusContext,
}

impl Clone for TestHarness {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl TestHarness {
    fn new() -> Self {
        TEST_VDOM.with(|vdom_cell| {
            let mut vdom_opt = vdom_cell.borrow_mut();
            if vdom_opt.is_none() {
                let mut vdom = VirtualDom::new(app);
                vdom.rebuild_in_place();
                *vdom_opt = Some(Box::new(vdom));
            }
        });
        Self {
            context: DioxusContext::new(),
        }
    }

    fn with_vdom<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        TEST_VDOM.with(|vdom_cell| {
            let vdom_opt = vdom_cell.borrow();
            vdom_opt.as_ref().unwrap().in_scope(ScopeId::ROOT, f)
        })
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

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        self.with_vdom(|| self.context.provide_global(value));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.with_vdom(|| self.context.use_global())
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
