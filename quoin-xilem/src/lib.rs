use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use xilem::tokio::runtime::Runtime;

/// Context that holds a tokio runtime and an optional update notifier.
#[derive(Clone)]
pub struct XilemContext {
    runtime: Arc<Runtime>,
    update_notifier: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
}

impl XilemContext {
    /// Create a new context with a tokio runtime handle.
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            update_notifier: Arc::new(Mutex::new(None)),
        }
    }

    /// Set a closure that will be called whenever any signal created from this context
    /// is mutated. This is typically used to request a UI rebuild.
    ///
    /// The closure must be `Send + Sync` because the context may be shared across threads.
    pub fn set_update_notifier(&self, notifier: impl Fn() + Send + Sync + 'static) {
        *self.update_notifier.lock().unwrap() = Some(Arc::new(notifier));
    }

    fn request_update(&self) {
        if let Some(notifier) = self.update_notifier.lock().unwrap().as_ref() {
            notifier();
        }
    }
}

impl ReactiveContext for XilemContext {
    type Signal<T: Clone + 'static> = XilemSignal<T>;
    type Executor = XilemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        XilemSignal {
            inner: Arc::new(std::sync::RwLock::new(initial)),
            context: self.clone(),
        }
    }

    fn executor(&self) -> Self::Executor {
        XilemExecutor {
            runtime: self.runtime.clone(),
        }
    }

    fn request_update(&self) {
        self.request_update();
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        // Stub: Xilem does not have a built-in context provider mechanism.
        None
    }
}

/// A thread‑safe reactive signal.
#[derive(Clone)]
pub struct XilemSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
    context: XilemContext,
}

impl<T: Clone + 'static> QuoinSignal<T> for XilemSignal<T> {
    fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.inner.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.inner.write().unwrap() = value;
        self.context.request_update();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
        self.context.request_update();
    }
}

/// Executor that spawns futures onto the tokio runtime.
#[derive(Clone)]
pub struct XilemExecutor {
    runtime: Arc<Runtime>,
}

impl Executor for XilemExecutor {
    type JoinHandle<T: Send + 'static> = XilemJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let handle = self.runtime.spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });
        XilemJoinHandle {
            handle: Some(handle),
            rx: Some(rx),
        }
    }
}

pub struct XilemJoinHandle<T> {
    handle: Option<tokio::task::JoinHandle<()>>,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for XilemJoinHandle<T> {
    fn abort(&self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
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

impl<T> Drop for XilemJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for XilemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XilemSignal")
            .field("value", &self.inner.read().map_err(|_| std::fmt::Error)?)
            .finish()
    }
}
