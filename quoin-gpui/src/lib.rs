//! GPUI adapter for quoin — reactive signals, async executors, and app bootstrap.
//!
//! This crate implements [`ReactiveContext`] for Zed's GPUI framework, enabling
//! framework-agnostic hooks and state to run inside GPUI windows and views.
//!
//! # Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`GpuiContext`] | Holds foreground/background executors and an update notifier. The primary entry point for creating signals. |
//! | [`GpuiSignal<T>`] | A `Clone + Send + Sync` signal backed by `Arc<RwLock<T>>`. Mutations automatically call `request_update()` to trigger view re-renders. |
//! | [`GpuiExecutor`] | Spawns tasks on GPUI's foreground or background executor. |
//! | [`GpuiJoinHandle<T>`] | A `JoinHandle` wrapping GPUI's `Task` with a oneshot channel for result retrieval. |
//!
//! # Creating a Context
//!
//! The idiomatic way is to convert from a GPUI `Context`:
//!
//! ```ignore
//! impl MyView {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         let ctx: GpuiContext = cx.into();
//!         let signal = ctx.create_signal(0u32);
//!         // ...
//!     }
//! }
//! ```
//!
//! # Automatic Re-rendering
//!
//! GPUI does not track signal reads automatically. You must connect the
//! context to your view's update cycle **once** during construction:
//!
//! ```ignore
//! ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
//! ```
//!
//! After this, any mutation to a `GpuiSignal` created from this context will
//! call `cx.notify()`, which schedules a re-render of the view.
//!
//! If you don't call `set_view_update_notifier`, signal mutations will still
//! update the underlying value but the view will **not** repaint.
//!
//! # Global State (Thread-Local)
//!
//! `provide_global` and `use_global` use a **thread-local** type-map. This means:
//!
//! - Globals are only visible on the thread that registered them.
//! - If your app opens windows on multiple threads, call `provide_global` on each.
//! - Each `use_global` call returns an **independent copy** of the value wrapped
//!   in a new `Arc<RwLock<T>>`. Mutations do not propagate back to other copies.
//!
//! # App Bootstrap
//!
//! The [`launch`] function wraps `gpui_platform::application().run()`. Combined
//! with the `run_app!` macro, a full GPUI app reduces to:
//!
//! ```ignore
//! run_app!(MyView);
//! ```
//!
//! # Minimum GPUI Version
//!
//! This crate tracks the `main` branch of the [Zed repository](https://github.com/zed-industries/zed).
//! Breaking changes in GPUI may propagate here without a semver bump.

use gpui::{AsyncWindowContext, BackgroundExecutor, Context, ForegroundExecutor, Task, WeakEntity};
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use send_wrapper::SendWrapper;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Type alias for the update notifier callback stored in `GpuiContext`.
type UpdateNotifier = Arc<dyn Fn() + Send + Sync>;

/// The GPUI context, which holds an optional notification callback.
/// The view is responsible for setting this callback to enable automatic UI updates.
#[derive(Clone)]
/// The GPUI reactive context.
///
/// Holds foreground/background executors and an update notifier.
/// Created with `GpuiContext::new(cx)` or `cx.into()`.
///
/// # Example
///
/// ```ignore
/// let ctx: GpuiContext = cx.into();
/// ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
/// let signal = ctx.create_signal(0);
/// ```
pub struct GpuiContext {
    pub foreground: SendWrapper<ForegroundExecutor>,
    pub background: Option<SendWrapper<BackgroundExecutor>>,
    /// Callback invoked when any signal created from this context is mutated.
    /// Stored as Arc<Mutex<...>> to satisfy Send + Sync bounds.
    update_notifier: Arc<Mutex<Option<UpdateNotifier>>>,
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

    fn notify_update(&self) {
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
        self.notify_update();
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        GLOBAL_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(TypeId::of::<T>(), Box::new(value));
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
                .cloned()
                .map(|v| GpuiSignal {
                    inner: Arc::new(std::sync::RwLock::new(v)),
                    context: self.clone(),
                })
        })
    }
}

#[derive(Clone)]
/// A GPUI-backed reactive signal.
///
/// Wraps an `Arc<RwLock<T>>` and notifies the context on mutation.
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
/// GPUI async executor.
///
/// Spawns tasks on the foreground or background executor.
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
    /// Abort the spawned task.
    ///
    /// # Limitations
    ///
    /// This is currently a **no-op**. The spawned thread runs to completion
    /// regardless of whether `abort()` is called. This is because:
    ///
    /// - The task runs on a dedicated `std::thread` with no cancellation token.
    /// - There is no built-in mechanism to interrupt a blocked `futures::executor::block_on`.
    ///
    /// If you need cancellation support, consider:
    /// - Using a `CancellationToken` pattern inside your future.
    /// - Switching to a tokio-based executor with proper task cancellation.
    ///
    /// This limitation will be addressed in a future version.
    /// Abort the spawned task.
    ///
    /// # Limitations
    ///
    /// This is currently a **no-op**. The spawned thread runs to completion
    /// regardless of whether `abort()` is called. This is because:
    ///
    /// - The task runs on a dedicated `std::thread` with no cancellation token.
    /// - There is no built-in mechanism to interrupt a blocked `futures::executor::block_on`.
    ///
    /// If you need cancellation support, consider:
    /// - Using a `CancellationToken` pattern inside your future.
    /// - Switching to a tokio-based executor with proper task cancellation.
    ///
    /// This limitation will be addressed in a future version.
    /// Abort the spawned task.
    ///
    /// # Limitations
    ///
    /// This is currently a **no-op**. The spawned thread runs to completion
    /// regardless of whether `abort()` is called. This is because:
    ///
    /// - The task runs on a dedicated `std::thread` with no cancellation token.
    /// - There is no built-in mechanism to interrupt a blocked `futures::executor::block_on`.
    ///
    /// If you need cancellation support, consider:
    /// - Using a `CancellationToken` pattern inside your future.
    /// - Switching to a tokio-based executor with proper task cancellation.
    ///
    /// This limitation will be addressed in a future version.
    /// Abort the spawned task.
    ///
    /// # Limitations
    ///
    /// This is currently a **no-op**. The spawned thread runs to completion
    /// regardless of whether `abort()` is called. This is because:
    ///
    /// - The task runs on a dedicated `std::thread` with no cancellation token.
    /// - There is no built-in mechanism to interrupt a blocked `futures::executor::block_on`.
    ///
    /// If you need cancellation support, consider:
    /// - Using a `CancellationToken` pattern inside your future.
    /// - Switching to a tokio-based executor with proper task cancellation.
    ///
    /// This limitation will be addressed in a future version.
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

/// Launch a GPUI application.
///
/// Wraps `gpui_platform::application().run()` so users don't need to
/// import `gpui_platform` directly.
///
/// # Example
/// ```rust,ignore
/// fn main() {
///     quoin::launch(|app_cx: &mut gpui::App| {
///         app_cx.open_window(gpui::WindowOptions::default(), |window, cx| {
///             // ...
///         }).unwrap();
///         app_cx.activate(true);
///     });
/// }
/// ```
pub fn launch<F>(f: F)
where
    F: FnOnce(&mut gpui::App) + 'static,
{
    gpui_platform::application().run(f);
}

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for GpuiSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuiSignal")
            .field("value", &self.inner.read().map_err(|_| std::fmt::Error)?)
            .finish()
    }
}
