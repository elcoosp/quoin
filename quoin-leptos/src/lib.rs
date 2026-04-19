use leptos::prelude::*;
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
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
        let (value, _set_value) = signal(SendWrapper::new(initial));
        LeptosSignal { value }
    }

    fn executor(&self) -> Self::Executor {
        LeptosExecutor
    }

    fn request_update(&self) {
        // Leptos is automatically reactive – nothing needed.
    }
}

#[derive(Clone)]
pub struct LeptosSignal<T: Clone + 'static> {
    value: ReadSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> Signal<T> for LeptosSignal<T> {
    fn get(&self) -> T {
        self.value.get().take()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with(|wrapper| f(&**wrapper))
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

        // Spawn on a dedicated thread with a blocking executor.
        // This avoids any_spawner's thread-local issues while ensuring
        // the future actually runs to completion.
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
impl<T: Send + 'static> JoinHandle<T> for LeptosJoinHandle<T> {
    fn abort(&self) {}
}
