use quoin_conformance::{SleepExt, define_conformance_tests};
use quoin_core::{Executor, JoinHandle, ReactiveContext};
use quoin_xilem::{XilemContext, XilemExecutor};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use xilem::tokio::runtime::Runtime;

#[derive(Clone)]
struct TestExecutor(XilemExecutor);

impl SleepExt for TestExecutor {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        futures_timer::Delay::new(duration)
    }
}

impl Executor for TestExecutor {
    type JoinHandle<T: Send + 'static> = <XilemExecutor as Executor>::JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }
}

struct TestHarness {
    context: XilemContext,
    _runtime: Arc<Runtime>,
}

impl Clone for TestHarness {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl TestHarness {
    fn new() -> Self {
        let runtime = Arc::new(Runtime::new().unwrap());
        Self {
            context: XilemContext::new(runtime.clone()),
            _runtime: runtime,
        }
    }
}

impl ReactiveContext for TestHarness {
    type Signal<T: Clone + 'static> = <XilemContext as ReactiveContext>::Signal<T>;
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

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        self.context.provide_global(value);
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.context.use_global()
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

// ---------------------------------------------------------------------------
// Additional test: JoinHandle::abort actually cancels the task
// ---------------------------------------------------------------------------
#[test]
fn test_join_handle_abort_cancels_task() {
    let harness = TestHarness::new();
    let executor = harness.executor();

    // Spawn a task that will never complete on its own
    let handle = executor.spawn(async {
        loop {
            futures_timer::Delay::new(Duration::from_secs(60)).await;
        }
    });

    // Immediately abort the task
    handle.abort();

    // Await the result via .into_future(). The oneshot receiver should
    // be cancelled because the underlying tokio task was aborted and the
    // sender dropped without sending a value.
    let result = futures::executor::block_on(handle.into_future());
    assert!(
        matches!(result, Err(futures::channel::oneshot::Canceled)),
        "Expected JoinHandle to be cancelled after abort, but got: {:?}",
        result
    );
}
