use leptos::prelude::*;
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Default)]
pub struct LeptosContext;

impl LeptosContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for LeptosContext {
    type Signal<T: Clone + 'static> = LeptosSignal<T>;
    type Executor = LeptosExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        let inner = RwSignal::new(SendWrapper::new(initial));
        LeptosSignal { inner }
    }

    fn executor(&self) -> Self::Executor {
        LeptosExecutor
    }

    fn request_update(&self) {
        // Leptos reactivity is automatic.
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        // Stub: Leptos context lookup returns RwSignal<T>, but our signal
        // wraps SendWrapper<T>. A full implementation would need a separate
        // global registry that stores SendWrapper-wrapped signals.
        None
    }
}

#[derive(Clone)]
pub struct LeptosSignal<T: Clone + 'static> {
    inner: RwSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> Signal<T> for LeptosSignal<T> {
    fn get(&self) -> T {
        self.inner.get().take()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.inner.with(|wrapper| f(&**wrapper))
    }

    fn set(&self, value: T) {
        self.inner.set(SendWrapper::new(value));
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(|wrapper| f(&mut **wrapper));
    }
}

#[derive(Clone)]
pub struct LeptosExecutor;

impl Executor for LeptosExecutor {
    type JoinHandle<T: Send + 'static> = LeptosJoinHandle<T>;

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

        LeptosJoinHandle { rx: Some(rx) }
    }
}

pub struct LeptosJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for LeptosJoinHandle<T> {
    fn abort(&self) {}
}

impl<T: Send + 'static> std::future::IntoFuture for LeptosJoinHandle<T> {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for LeptosSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeptosSignal")
            .field("value", &self.inner.get())
            .finish()
    }
}
