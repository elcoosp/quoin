use floem_reactive::{ReadSignal, RwSignal, SignalGet, SignalWith};
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct FloemContext;

impl FloemContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for FloemContext {
    type Signal<T: Clone + 'static> = FloemSignal<T>;
    type Executor = FloemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        FloemSignal::new(initial)
    }

    fn executor(&self) -> Self::Executor {
        FloemExecutor
    }

    fn request_update(&self) {
        // Floem's reactivity is automatic via signal subscriptions.
    }
}

#[derive(Clone)]
pub struct FloemSignal<T: Clone + 'static> {
    value: ReadSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> FloemSignal<T> {
    fn new(initial: T) -> Self {
        let rw_signal = RwSignal::new(SendWrapper::new(initial));
        Self {
            value: rw_signal.read_only(),
        }
    }
}

impl<T: Clone + 'static> Signal<T> for FloemSignal<T> {
    fn get(&self) -> T {
        // SignalGet::get() returns SendWrapper<T>, then we deref and clone
        (*self.value.get()).clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with(|wrapper| f(&**wrapper))
    }
}

#[derive(Clone)]
pub struct FloemExecutor;

impl Executor for FloemExecutor {
    type JoinHandle<T: Send + 'static> = FloemJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        std::thread::spawn(move || {
            let result = futures::executor::block_on(future);
            let _ = tx.send(result);
        });

        FloemJoinHandle { rx: Some(rx) }
    }
}

pub struct FloemJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for FloemJoinHandle<T> {
    fn abort(&self) {}
}

impl<T: Send + 'static> std::future::IntoFuture for FloemJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            if let Some(rx) = self.rx.take() {
                rx.await
            } else {
                unreachable!("Receiver should be set")
            }
        })
    }
}
