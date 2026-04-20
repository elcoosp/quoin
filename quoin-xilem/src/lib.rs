use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct XilemContext;

impl XilemContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for XilemContext {
    type Signal<T: Clone + 'static> = XilemSignal<T>;
    type Executor = XilemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        XilemSignal::new(initial)
    }

    fn executor(&self) -> Self::Executor {
        XilemExecutor
    }

    fn request_update(&self) {
        // Xilem's reactivity is driven by app state changes.
    }
}

#[derive(Clone)]
pub struct XilemSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
}

impl<T: Clone + 'static> XilemSignal<T> {
    fn new(value: T) -> Self {
        Self {
            inner: Arc::new(std::sync::RwLock::new(value)),
        }
    }
}

impl<T: Clone + 'static> Signal<T> for XilemSignal<T> {
    fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        let guard = self.inner.read().unwrap();
        f(&guard)
    }

    fn set(&self, value: T) {
        *self.inner.write().unwrap() = value;
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
    }
}

#[derive(Clone)]
pub struct XilemExecutor;

impl Executor for XilemExecutor {
    type JoinHandle<T: Send + 'static> = XilemJoinHandle<T>;

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

        XilemJoinHandle { rx: Some(rx) }
    }
}

pub struct XilemJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for XilemJoinHandle<T> {
    fn abort(&self) {}
}

impl<T: Send + 'static> std::future::IntoFuture for XilemJoinHandle<T> {
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
