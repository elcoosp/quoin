use dioxus::prelude::*;
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
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
        // CopyValue is the base storage for all signals in Dioxus 0.7.
        // Despite the name, it does NOT require T: Copy.
        let signal = ReadSignal::new(CopyValue::new(initial));
        DioxusSignal { signal }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus is automatically reactive
    }
}

#[derive(Clone, Copy)]
pub struct DioxusSignal<T: Clone + 'static> {
    signal: ReadSignal<T>,
}

impl<T: Clone + 'static> Signal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.signal.read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&*self.signal.read())
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

        std::thread::spawn(move || {
            let result = futures::executor::block_on(future);
            let _ = tx.send(result);
        });

        DioxusJoinHandle { rx: Some(rx) }
    }
}

pub struct DioxusJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for DioxusJoinHandle<T> {
    fn abort(&self) {}
}
