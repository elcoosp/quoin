use dioxus::prelude::*;
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::future::Future;

#[derive(Clone)]
pub struct DioxusContext;

impl DioxusContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for DioxusContext {
    type Signal<T: Clone + 'static> = DioxusSignal<T>;
    type Executor = DioxusExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        let signal = use_signal(|| initial);
        DioxusSignal { signal }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus is automatically reactive
    }
}

#[derive(Clone)]
pub struct DioxusSignal<T: Clone + 'static> {
    signal: dioxus::prelude::Signal<T>,
}

impl<T: Clone + 'static> Signal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.signal.read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.signal.read())
    }
}

#[derive(Clone)]
pub struct DioxusExecutor;

impl Executor for DioxusExecutor {
    type JoinHandle<T: Send + 'static> = DioxusJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let task = dioxus_core::spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });
        DioxusJoinHandle {
            task: SendWrapper::new(task),
            rx: Some(rx),
        }
    }
}

pub struct DioxusJoinHandle<T> {
    task: SendWrapper<dioxus_core::Task>,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

unsafe impl<T> Send for DioxusJoinHandle<T> {}
unsafe impl<T> Sync for DioxusJoinHandle<T> {}

impl<T: Send + 'static> JoinHandle<T> for DioxusJoinHandle<T> {
    fn abort(&self) {
        self.task.cancel();
    }
}

impl<T> Drop for DioxusJoinHandle<T> {
    fn drop(&mut self) {
        self.task.cancel();
    }
}
