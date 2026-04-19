use floem_reactive::{ReadSignal, RwSignal};
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
pub struct FloemSignal<T: Clone + 'static> {
    value: ReadSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> Signal<T> for FloemSignal<T> {
    fn get(&self) -> T {
        todo!()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        todo!()
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
        todo!()
    }
}

pub struct FloemJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for FloemJoinHandle<T> {
    fn abort(&self) {}
}
