//! Xilem adapter for quoin — reactive signals, async executors, and app lifecycle.
//!
//! This crate implements [`ReactiveContext`] for the [Xilem](https://github.com/linebender/xilem)
//! UI toolkit, enabling framework-agnostic hooks and state to run inside
//! Xilem views.
//!
//! # Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`XilemContext`] | Holds a `tokio::runtime::Runtime` and an optional update notifier. |
//! | [`XilemSignal<T>`] | A `Clone + Send + Sync` signal backed by `Arc<RwLock<T>>`. |
//! | [`XilemExecutor`] | Spawns tasks on the provided tokio runtime. |
//! | [`XilemJoinHandle<T>`] | A `JoinHandle` wrapping a `tokio::task::JoinHandle` with abort support. |
//!
//! # Creating a Context
//!
//! Xilem requires an explicit async runtime. Create one and pass it to the context:
//!
//! ```ignore
//! use std::sync::Arc;
//! use xilem::tokio::runtime::Runtime;
//!
//! let runtime = Arc::new(Runtime::new().unwrap());
//! let ctx = XilemContext::new(runtime);
//! let count = ctx.create_signal(0i32);
//! ```
//!
//! # Automatic Re-rendering
//!
//! Like GPUI, Xilem does not automatically track signal reads. You must set an
//! update notifier that requests a UI rebuild when signals change:
//!
//! ```ignore
//! ctx.set_update_notifier(|| {
//!     println!("Signal changed – request UI rebuild here");
//! });
//! ```
//!
//! The exact mechanism for triggering a rebuild depends on your Xilem integration
//! (e.g., `EventLoop::set_idle_callback` or re-calling `Xilem::new`).
//!
//! # Signal Threading Model
//!
//! `XilemSignal<T>` is backed by `Arc<RwLock<T>>`, making it truly `Send + Sync`:
//!
//! - Safe to read/write from any thread.
//! - Mutations call `request_update()` which invokes the notifier callback.
//! - The notifier callback itself must be `Send + Sync`.
//!
//! # Async Executor
//!
//! Unlike other adapters, `XilemExecutor` uses a real tokio runtime (not blocking
//! threads). This makes it suitable for actual async I/O:
//!
//! ```ignore
//! let executor = ctx.executor();
//! executor.spawn(async {
//!     let response = reqwest::get("https://example.com").await?;
//!     // ...
//! });
//! ```
//!
//! # Global State (Thread-Local Store, Shared Values)
//!
//! Uses the shared thread-local type-map from `quoin_core::globals` (same
//! pattern as GPUI/Floem, implemented once). Globals are only visible on the
//! registering thread. The stored value is the `XilemSignal` itself, so every
//! `use_global` consumer shares one `Arc<RwLock>` cell and mutations
//! propagate between consumers (unified with the other adapters).
//!
//! The adapter also implements [`quoin_core::LocalExecutor`]: `spawn_local`
//! queues `!Send` futures on the current thread, driven by
//! `quoin_core::local::run_local_tasks()`.

use quoin_core::{Executor, ReactiveContext};
use std::future::Future;
use std::sync::{Arc, Mutex};
use xilem::tokio::runtime::Runtime;

type UpdateNotifier = Arc<dyn Fn() + Send + Sync>;

/// Context that holds a tokio runtime and an optional update notifier.
#[derive(Clone)]
pub struct XilemContext {
    runtime: Arc<Runtime>,
    update_notifier: Arc<Mutex<Option<UpdateNotifier>>>,
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
    ///
    /// # Panics
    /// Panics if the notifier mutex is poisoned (a prior panic while holding it).
    pub fn set_update_notifier(&self, notifier: impl Fn() + Send + Sync + 'static) {
        *self.update_notifier.lock().unwrap() = Some(Arc::new(notifier));
    }

    /// Invoke the update notifier, if one is registered.
    fn notify_update(&self) {
        if let Some(notifier) = self.update_notifier.lock().unwrap().as_ref() {
            notifier();
        }
    }

    /// Builds a signal whose notifier routes through this context. The
    /// shared `NotifierSignal` cell-plus-notifier implementation (batch
    /// suppression, single-lock `set_if_eq`) lives in `quoin-core`.
    fn make_signal<T: Clone + 'static>(&self, initial: T) -> XilemSignal<T> {
        let context = self.clone();
        quoin_core::NotifierSignal::new(initial, move || context.notify_update())
    }
}

impl ReactiveContext for XilemContext {
    type Signal<T: Clone + 'static> = XilemSignal<T>;
    type Executor = XilemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.make_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        XilemExecutor {
            runtime: self.runtime.clone(),
        }
    }

    fn request_update(&self) {
        // Explicitly call the inherent method: a bare `self.request_update()`
        // would recurse into this trait method if the inherent one were ever
        // removed, and method resolution silently prefers the inherent method.
        Self::notify_update(self);
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        // Store the signal itself (XilemSignal is Send + Sync) so every
        // consumer shares the same `Arc<RwLock>`: mutations propagate between
        // consumers (matching the other adapters; previously each use_global
        // returned an independent copy).
        quoin_core::globals::insert::<Self, XilemSignal<T>>(self.make_signal(value));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        quoin_core::globals::get::<Self, XilemSignal<T>>()
    }
}

/// A thread‑safe reactive signal.
///
/// This is [`quoin_core::NotifierSignal`] under a historical name: the
/// `Arc<RwLock<T>>` cell plus update-notifier implementation (including
/// batch-aware notification and the single-lock `set_if_eq` fast path) is
/// implemented once in `quoin-core` and shared with the GPUI adapter.
pub type XilemSignal<T> = quoin_core::NotifierSignal<T>;

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
        quoin_core::SharedJoinHandle::new(XilemTask(Mutex::new(Some(handle))), rx)
    }
}

/// The tokio task wrapped for the shared handle.
///
/// Tokio tasks are independent of their handles: `abort` takes the handle
/// out and calls tokio's abort; `detach` is a no-op because dropping a tokio
/// `JoinHandle` never cancels the task.
pub struct XilemTask(Mutex<Option<tokio::task::JoinHandle<()>>>);

impl quoin_core::RawTask for XilemTask {
    fn abort(&self) {
        // Take the handle while holding the lock, abort after the guard is
        // gone: tokio's abort may synchronously interact with the task.
        let handle = self.0.lock().unwrap().take();
        if let Some(handle) = handle {
            handle.abort();
        }
    }

    fn detach(&self) {}
}

/// Handle for a task spawned on the tokio runtime backing `XilemContext`.
///
/// Dropping it without awaiting or detaching cancels the task (structured
/// concurrency); `abort` and `detach` map to the tokio equivalents.
///
/// This is [`quoin_core::SharedJoinHandle`] over [`XilemTask`] under a
/// historical name: the await / abort / detach / abort-on-drop lifecycle is
/// implemented once in `quoin-core` and shared with the GPUI adapter (and
/// with the pooled `Abortable` adapters via `AbortJoinHandle`).
pub type XilemJoinHandle<T> = quoin_core::SharedJoinHandle<T, XilemTask>;

impl quoin_core::LocalExecutor for XilemExecutor {
    type JoinHandle<T: 'static> = quoin_core::LocalTask<T>;

    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: std::future::Future + 'static,
        F::Output: 'static,
    {
        // Queued on the current thread; drive with
        // `quoin_core::local::run_local_tasks()`.
        quoin_core::spawn_local_task(future)
    }
}
