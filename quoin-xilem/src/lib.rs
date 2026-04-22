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
//! # Global State (Thread-Local)
//!
//! Uses a thread-local type-map (same pattern as GPUI/Floem). Globals are only
//! visible on the registering thread. Each `use_global` creates an independent copy.

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

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        GLOBAL_STORE.with(|store| {
            store.borrow_mut().insert(TypeId::of::<T>(), Box::new(value));
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
                .cloned()
                .map(|v| XilemSignal {
                    inner: Arc::new(std::sync::RwLock::new(v)),
                    context: self.clone(),
                })
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
