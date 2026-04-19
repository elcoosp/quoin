use gpui::{Context, ForegroundExecutor, Task};
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct GpuiContext {
    foreground: SendWrapper<ForegroundExecutor>,
}

impl GpuiContext {
    pub fn new<T: 'static>(cx: &mut Context<T>) -> Self {
        Self {
            foreground: SendWrapper::new(cx.foreground_executor().clone()),
        }
    }
    /// New constructor that directly takes a `ForegroundExecutor`.
    /// This is ideal for test environments where we don't have a `Context`.
    pub fn from_executor(foreground: ForegroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
        }
    }
}

impl ReactiveContext for GpuiContext {
    type Signal<T: Clone + 'static> = GpuiSignal<T>;
    type Executor = GpuiExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        GpuiSignal {
            inner: Arc::new(std::sync::RwLock::new(initial)),
        }
    }

    fn executor(&self) -> Self::Executor {
        GpuiExecutor {
            foreground: self.foreground.clone(),
        }
    }

    fn request_update(&self) {
        // GPUI requires Context to notify; placeholder
    }
}

#[derive(Clone)]
pub struct GpuiSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
}

impl<T: Clone + 'static> Signal<T> for GpuiSignal<T> {
    fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        let guard = self.inner.read().unwrap();
        f(&guard)
    }
}

#[derive(Clone)]
pub struct GpuiExecutor {
    foreground: SendWrapper<ForegroundExecutor>,
}

impl Executor for GpuiExecutor {
    type JoinHandle<T: Send + 'static> = GpuiJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let task = self.foreground.spawn(async move {
            let result = future.await;
            let _ = tx.send(result);
        });
        GpuiJoinHandle {
            task: Some(task),
            rx: Some(rx),
        }
    }
}

pub struct GpuiJoinHandle<T> {
    task: Option<Task<()>>,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for GpuiJoinHandle<T> {
    fn abort(&self) {}
}

impl<T> Drop for GpuiJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            drop(task);
        }
    }
}
