use gpui::{AsyncWindowContext, BackgroundExecutor, Context, ForegroundExecutor, Task, WeakEntity};
use quoin::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// The GPUI context, which holds an optional notification callback.
/// The view is responsible for setting this callback to enable automatic UI updates.
#[derive(Clone)]
pub struct GpuiContext {
    pub foreground: SendWrapper<ForegroundExecutor>,
    pub background: Option<SendWrapper<BackgroundExecutor>>,
    // Callback invoked when any signal created from this context is mutated.
    // Stored as Arc<Mutex<...>> to satisfy Send + Sync bounds.
    update_notifier: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
}

impl GpuiContext {
    /// Create a new context. Prefer using `cx.into()` for brevity.
    pub fn new<T: 'static>(cx: &mut Context<T>) -> Self {
        cx.into()
    }

    /// Set the notification callback. This should be a closure that triggers a view repaint,
    /// e.g., `move || cx.notify()`.
    ///
    /// The closure must be `Send + Sync` because `GpuiContext` may be shared across threads.
    pub fn set_update_notifier(&self, notifier: impl Fn() + Send + Sync + 'static) {
        *self.update_notifier.lock().unwrap() = Some(Arc::new(notifier));
    }

    /// Convenience method that connects a view to reactive updates.
    ///
    /// This sets up a notifier that automatically refreshes the given view whenever
    /// any signal created from this context changes.
    pub fn set_view_update_notifier<V: 'static>(
        &self,
        weak_view: WeakEntity<V>,
        async_window: AsyncWindowContext,
    ) {
        // Wrap the AsyncWindowContext to make it Send + Sync.
        let async_window = SendWrapper::new(async_window);
        self.set_update_notifier(move || {
            let async_window = async_window.clone();
            let weak_view = weak_view.clone();
            async_window
                .spawn(async move |cx| {
                    if let Some(view) = weak_view.upgrade() {
                        view.update(cx, |_, cx| cx.notify());
                    }
                })
                .detach();
        });
    }

    /// Create a context from an existing foreground executor (useful for tests).
    pub fn from_executor(foreground: ForegroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
            background: None,
            update_notifier: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a context with both foreground and background executors (useful for tests).
    pub fn for_test(foreground: ForegroundExecutor, background: BackgroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
            background: Some(SendWrapper::new(background)),
            update_notifier: Arc::new(Mutex::new(None)),
        }
    }

    fn request_update(&self) {
        if let Some(notifier) = self.update_notifier.lock().unwrap().as_ref() {
            notifier();
        }
    }
}

/// Allow creating a `GpuiContext` directly from a GPUI `Context`.
impl<T: 'static> From<&mut Context<'_, T>> for GpuiContext {
    fn from(cx: &mut Context<'_, T>) -> Self {
        Self {
            foreground: SendWrapper::new(cx.foreground_executor().clone()),
            background: None,
            update_notifier: Arc::new(Mutex::new(None)),
        }
    }
}

impl ReactiveContext for GpuiContext {
    type Signal<T: Clone + 'static> = GpuiSignal<T>;
    type Executor = GpuiExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        GpuiSignal {
            inner: Arc::new(std::sync::RwLock::new(initial)),
            context: self.clone(),
        }
    }

    fn executor(&self) -> Self::Executor {
        GpuiExecutor {
            foreground: self.foreground.clone(),
            background: self.background.clone(),
        }
    }

    fn request_update(&self) {
        self.request_update();
    }
}

#[derive(Clone)]
pub struct GpuiSignal<T: Clone + 'static> {
    inner: Arc<std::sync::RwLock<T>>,
    context: GpuiContext,
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
        self.context.request_update();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
        self.context.request_update();
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
    /// Returns a future that completes after the specified duration.
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
