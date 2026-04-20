// quoin-gpui/src/lib.rs
use gpui::{BackgroundExecutor, Context, ForegroundExecutor, Task};
use quoin::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct GpuiContext {
    foreground: SendWrapper<ForegroundExecutor>,
    background: Option<SendWrapper<BackgroundExecutor>>,
}

impl GpuiContext {
    pub fn new<T: 'static>(_cx: &mut Context<T>) -> Self {
        Self {
            foreground: SendWrapper::new(_cx.foreground_executor().clone()),
            background: None,
        }
    }

    pub fn from_executor(foreground: ForegroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
            background: None,
        }
    }

    pub fn for_test(foreground: ForegroundExecutor, background: BackgroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
            background: Some(SendWrapper::new(background)),
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
            background: self.background.clone(),
        }
    }

    fn request_update(&self) {
        // No automatic update in GPUI – user must call cx.notify() manually.
    }
}

#[derive(Clone)]
pub struct GpuiSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
}

impl<T: Clone + 'static> QuoinSignal<T> for GpuiSignal<T> {
    fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.inner.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.inner.write().unwrap() = value;
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
    }
}

#[derive(Clone)]
pub struct GpuiExecutor {
    foreground: SendWrapper<ForegroundExecutor>,
    background: Option<SendWrapper<BackgroundExecutor>>,
}

impl Executor for GpuiExecutor {
    type JoinHandle<T: Send + 'static> = GpuiJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        let task = if let Some(bg) = &self.background {
            bg.spawn(async move {
                let result = future.await;
                let _ = tx.send(result);
            })
        } else {
            self.foreground.spawn(async move {
                let result = future.await;
                let _ = tx.send(result);
            })
        };

        GpuiJoinHandle {
            task: Some(task),
            rx: Some(rx),
        }
    }
}

impl GpuiExecutor {
    pub fn sleep(&self, duration: std::time::Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        if let Some(bg) = &self.background {
            let bg = bg.clone();
            Box::pin(async move {
                bg.deref().timer(duration).await;
            })
        } else {
            let fg = self.foreground.clone();
            Box::pin(async move {
                let scheduler = fg.deref().scheduler_executor().scheduler().clone();
                scheduler.timer(duration).await;
            })
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

impl<T: Send + 'static> std::future::IntoFuture for GpuiJoinHandle<T> {
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

impl<T> Drop for GpuiJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            drop(task);
        }
    }
}
