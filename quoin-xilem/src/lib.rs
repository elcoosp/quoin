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
//! Uses a thread-local type-map (same pattern as GPUI/Floem). Globals are only
//! visible on the registering thread. The stored value is the `XilemSignal`
//! itself, so every `use_global` consumer shares one `Arc<RwLock>` cell and
//! mutations propagate between consumers (unified with the other adapters).
//!
//! The adapter also implements [`quoin_core::LocalExecutor`]: `spawn_local`
//! queues `!Send` futures on the current thread, driven by
//! `quoin_core::local::run_local_tasks()`.

use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use xilem::tokio::runtime::Runtime;

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

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
    pub fn set_update_notifier(&self, notifier: impl Fn() + Send + Sync + 'static) {
        *self.update_notifier.lock().unwrap() = Some(Arc::new(notifier));
    }

    /// Invoke the update notifier, if one is registered.
    fn notify_update(&self) {
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
        GLOBAL_STORE.with(|store| {
            store.borrow_mut().insert(
                TypeId::of::<T>(),
                Box::new(XilemSignal {
                    inner: Arc::new(std::sync::RwLock::new(value)),
                    context: self.clone(),
                }),
            );
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<XilemSignal<T>>())
                .cloned()
        })
    }
}

/// A thread‑safe reactive signal.
#[derive(Clone)]
/// Xilem-backed reactive signal.
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
        // Inside `quoin_core::batch` the final `request_update` is issued by
        // the batch itself; per-write notifies would be redundant work.
        if quoin_core::batch::should_notify() {
            self.context.request_update();
        }
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
        if quoin_core::batch::should_notify() {
            self.context.request_update();
        }
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
            detached: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

pub struct XilemJoinHandle<T> {
    handle: Option<tokio::task::JoinHandle<()>>,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: std::sync::atomic::AtomicBool,
}

impl<T: Send + 'static> JoinHandle<T> for XilemJoinHandle<T> {
    /// Abort the spawned task.
    ///
    /// Uses tokio's native task cancellation: the future stops at its next
    /// suspension point. Any await of this handle then resolves with
    /// `Err(Canceled)`.
    fn abort(&self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }

    /// Detach the task so it runs to completion even if this handle is
    /// dropped. The task's result is discarded.
    fn detach(self) {
        self.detached
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl<T: Send + 'static> std::future::IntoFuture for XilemJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            if let Some(rx) = self.rx.take() {
                // drop-after-await must not abort the task.
                self.detached
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                rx.await
            } else {
                Err(futures::channel::oneshot::Canceled)
            }
        })
    }
}

impl<T> Drop for XilemJoinHandle<T> {
    fn drop(&mut self) {
        // Structured concurrency: dropping the handle without awaiting or
        // detaching cancels the underlying tokio task.
        if !self.detached.load(std::sync::atomic::Ordering::SeqCst)
            && let Some(handle) = self.handle.take()
        {
            handle.abort();
        }
    }
}

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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for XilemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XilemSignal")
            .field(
                "value",
                &self.inner.read().unwrap_or_else(|e| e.into_inner()),
            )
            .finish()
    }
}
