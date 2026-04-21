use dioxus::prelude::*;
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use std::cell::RefCell;
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
        DioxusSignal {
            inner: RefCell::new(Signal::new(initial)),
        }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus reactivity is automatic.
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        // Stub: Dioxus context retrieval requires being inside a component scope
        // and the exact API depends on how the context was provided.
        // A full implementation would use dioxus::prelude::use_context.
        None
    }
}

#[derive(Clone)]
pub struct DioxusSignal<T: Clone + 'static> {
    inner: RefCell<Signal<T>>,
}

impl<T: Clone + 'static> QuoinSignal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.inner.borrow().read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.inner.borrow().read())
    }

    fn set(&self, value: T) {
        *self.inner.borrow_mut().write() = value;
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.borrow_mut().write());
    }
}

// Executor (unchanged)
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for DioxusSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DioxusSignal")
            .field("value", &self.inner.borrow().read())
            .finish()
    }
}
