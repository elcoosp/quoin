use dioxus::prelude::*;
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use std::future::Future;
use std::pin::Pin;

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
        // In Dioxus 0.7, Signal::new returns a Signal<T> that is both readable and writable.
        let signal = dioxus::prelude::Signal::new(initial);
        DioxusSignal { signal }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus reactivity is automatic.
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

impl<T: Send + 'static> std::future::IntoFuture for DioxusJoinHandle<T> {
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
