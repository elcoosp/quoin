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
        todo!()
    }

    fn executor(&self) -> Self::Executor {
        todo!()
    }

    fn request_update(&self) {
        todo!()
    }
}

#[derive(Clone)]
pub struct XilemSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
}

impl<T: Clone + 'static> Signal<T> for XilemSignal<T> {
    fn get(&self) -> T {
        todo!()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        todo!()
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
        todo!()
    }
}

pub struct XilemJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for XilemJoinHandle<T> {
    fn abort(&self) {}
}
