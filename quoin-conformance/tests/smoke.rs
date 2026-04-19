use quoin::ReactiveContext;
use quoin_conformance::ReactiveContextConformance;

#[derive(Clone)]
struct DummyContext;

impl ReactiveContext for DummyContext {
    type Signal<T: Clone + 'static> = DummySignal<T>;
    type Executor = DummyExecutor;

    fn create_signal<T: Clone + 'static>(&self, _initial: T) -> Self::Signal<T> {
        unimplemented!()
    }

    fn executor(&self) -> Self::Executor {
        DummyExecutor
    }

    fn request_update(&self) {
        unimplemented!()
    }
}

struct DummySignal<T>(std::marker::PhantomData<T>);

impl<T> Copy for DummySignal<T> {}
impl<T> Clone for DummySignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + 'static> quoin::Signal<T> for DummySignal<T> {
    fn get(&self) -> T {
        unimplemented!()
    }
    fn with<U>(&self, _f: impl FnOnce(&T) -> U) -> U {
        unimplemented!()
    }
}

#[derive(Clone)]
struct DummyExecutor;

impl quoin::Executor for DummyExecutor {
    type JoinHandle<T: Send + 'static> = DummyJoinHandle<T>;

    fn spawn<F>(&self, _future: F) -> Self::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        DummyJoinHandle(std::marker::PhantomData)
    }
}

struct DummyJoinHandle<T>(std::marker::PhantomData<T>);

unsafe impl<T> Sync for DummyJoinHandle<T> {}

impl<T: Send + 'static> quoin::JoinHandle<T> for DummyJoinHandle<T> {
    fn abort(&self) {}
}

fn _ensure_trait_bounds() {
    fn _assert_conformance<T: ReactiveContextConformance>() {}
}
