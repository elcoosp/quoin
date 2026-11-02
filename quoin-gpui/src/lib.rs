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
//! # Global State (Thread-Local Store, Shared Values)
//!
//! `provide_global` and `use_global` use a **thread-local** type-map. This means:
//!
//! - Globals are only visible on the thread that registered them.
//! - If your app opens windows on multiple threads, call `provide_global` on each.
//! - The stored value is the `GpuiSignal` itself, so every `use_global`
//!   consumer shares one `Arc<RwLock<T>>` cell: mutations propagate between
//!   consumers and notify the view (unified with the other adapters).
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
use quoin_core::join::SharedJoinHandle;
use quoin_core::{Executor, RawTask, ReactiveContext};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

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
    /// GPUI foreground (UI-thread) executor, wrapped for `Send`.
    pub foreground: SendWrapper<ForegroundExecutor>,
    /// Optional background executor; `None` when created from a context
    /// that has no background pool available.
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
    ///
    /// # Panics
    ///
    /// Panics if the notifier mutex is poisoned (a prior panic while
    /// notifying).
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

    /// Connects the context to a whole-window refresh: any signal mutation
    /// (or `request_update()` call) repaints **every** entity in the window,
    /// mirroring classic "refresh the window on state change" adapters.
    ///
    /// This is the notifier multi-entity apps (a root view plus independent
    /// panel views) need: with [`Self::set_view_update_notifier`] only the
    /// single wired view repaints.
    pub fn set_window_refresh_notifier(&self, async_window: AsyncWindowContext) {
        // Wrap the AsyncWindowContext to make it Send + Sync.
        let async_window = SendWrapper::new(async_window);
        self.set_update_notifier(move || {
            let async_window = async_window.clone();
            async_window
                .spawn(async move |cx| {
                    cx.update(|window, _| window.refresh()).ok();
                })
                .detach();
        });
    }

    /// Create a context from an existing foreground executor (useful for tests).
    #[must_use]
    pub fn from_executor(foreground: ForegroundExecutor) -> Self {
        Self {
            foreground: SendWrapper::new(foreground),
            background: None,
            update_notifier: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a context with both foreground and background executors (useful for tests).
    #[must_use]
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

    /// Builds a signal wired to this context's notifier slot. The signal
    /// stores only the shared slot (not the whole context), which keeps it
    /// `Send + Sync`.
    fn make_signal<T: Clone + 'static>(&self, initial: T) -> GpuiSignal<T> {
        let notifier_slot = Arc::clone(&self.update_notifier);
        quoin_core::NotifierSignal::new(initial, move || {
            if let Some(notifier) = notifier_slot.lock().unwrap().as_ref() {
                notifier();
            }
        })
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
        self.make_signal(initial)
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
        // Store the signal itself (GpuiSignal is Send + Sync: it holds only
        // an Arc<RwLock<T>> and the notifier arc) so every consumer shares
        // the same cell: mutations propagate between consumers and notify the
        // view (unified with the other adapters).
        quoin_core::globals::insert::<Self, GpuiSignal<T>>(self.make_signal(value));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        quoin_core::globals::get::<Self, GpuiSignal<T>>()
    }
}

/// A GPUI-backed reactive signal.
///
/// Wraps an `Arc<RwLock<T>>` and notifies the context's update callback on
/// mutation. The signal deliberately stores only the notifier handle (not the
/// whole [`GpuiContext`]), which keeps it `Send + Sync` — required for the
/// shared global store.
///
/// This is [`quoin_core::NotifierSignal`] under a historical name: the
/// cell-plus-notifier implementation (including the batch-aware notification
/// and the single-lock `set_if_eq` fast path) is implemented once in
/// `quoin-core` and shared with the Xilem adapter.
pub type GpuiSignal<T> = quoin_core::NotifierSignal<T>;

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

        SharedJoinHandle::new(GpuiTask(Mutex::new(Some(task))), rx)
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

/// The GPUI task wrapped for the shared handle.
///
/// Dropping the `Task` cancels it, so `abort` (and the handle's abort-on-drop
/// rule) takes the task out of the slot and drops it; `detach` releases
/// ownership via `Task::detach`.
pub struct GpuiTask(Mutex<Option<Task<()>>>);

impl RawTask for GpuiTask {
    fn abort(&self) {
        // Dropping the `Task` cancels it immediately (see GPUI docs).
        drop(self.0.lock().unwrap().take());
    }

    fn detach(&self) {
        // In GPUI the `Task` handle *owns* the task: dropping it cancels the
        // task. `Task::detach` releases that ownership into the scheduler so
        // the future keeps running. Take the task while holding the lock,
        // then detach after the guard is gone.
        let task = self.0.lock().unwrap().take();
        if let Some(task) = task {
            task.detach();
        }
    }
}

/// Handle for a task spawned on GPUI's foreground or background executor.
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task
/// (structured concurrency).
///
/// This is [`quoin_core::SharedJoinHandle`] over [`GpuiTask`] under a
/// historical name: the await / abort / detach / abort-on-drop lifecycle is
/// implemented once in `quoin-core` and shared with the Xilem adapter (and
/// with the pooled `Abortable` adapters via `AbortJoinHandle`).
pub type GpuiJoinHandle<T> = SharedJoinHandle<T, GpuiTask>;

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
