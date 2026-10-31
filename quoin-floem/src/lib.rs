//! Floem adapter for quoin — reactive signals and async executors.
//!
//! This crate implements [`ReactiveContext`] for the [Floem](https://github.com/lapce/floem)
//! UI framework, enabling framework-agnostic hooks and state to run inside
//! Floem views.
//!
//! # Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`FloemContext`] | A zero-sized context. Creates signals via Floem's `RwSignal`. |
//! | [`FloemSignal<T>`] | A `Clone + Send + Sync` signal backed by `RwSignal<SendWrapper<T>>`. |
//! | [`FloemExecutor`] | Spawns futures on worker threads with cooperative abort support. |
//! | [`FloemJoinHandle<T>`] | A `JoinHandle` with abort/detach and result retrieval. |
//!
//! # Creating a Context
//!
//! ```ignore
//! fn app_view() -> impl IntoView {
//!     let ctx = FloemContext::new();
//!     let count = ctx.create_signal(0u32);
//!     // ...
//! }
//! ```
//!
//! # Signal Threading Model
//!
//! Like the Leptos adapter, Floem signals are not inherently `Send`. Values are
//! wrapped in [`SendWrapper`](https://docs.rs/send_wrapper) to satisfy quoin's bounds:
//!
//! - `FloemSignal<T>` is `Clone + Send + Sync`.
//! - The underlying `RwSignal` must be accessed on the main thread.
//! - Safe for storing in structs that cross thread boundaries, but **do not**
//!   call `.get()` / `.set()` from a background thread.
//!
//! # Global State (Thread-Local)
//!
//! `provide_global` and `use_global` use a **thread-local** type-map (same pattern
//! as the GPUI adapter). Globals are only visible on the registering thread.
//!
//! Each `use_global` call creates an independent copy of the value in a new signal.

use floem_reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone, Default)]
/// Floem reactive context.
pub struct FloemContext;

impl FloemContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for FloemContext {
    type Signal<T: Clone + 'static> = FloemSignal<T>;
    type Executor = FloemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        FloemSignal::new(initial)
    }

    fn executor(&self) -> Self::Executor {
        FloemExecutor
    }

    fn request_update(&self) {
        // Floem's reactivity is automatic.
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
                .map(|v| FloemSignal::new(v))
        })
    }
}

#[derive(Clone)]
/// Floem-backed reactive signal.
pub struct FloemSignal<T: Clone + 'static> {
    inner: RwSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> FloemSignal<T> {
    fn new(initial: T) -> Self {
        Self {
            inner: RwSignal::new(SendWrapper::new(initial)),
        }
    }
}

impl<T: Clone + 'static> Signal<T> for FloemSignal<T> {
    fn get(&self) -> T {
        (*self.inner.get()).clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.inner.with(|wrapper| f(&**wrapper))
    }

    fn set(&self, value: T) {
        self.inner.set(SendWrapper::new(value));
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(|wrapper| f(&mut **wrapper));
    }
}

/// Floem async executor.
///
/// Futures run on dedicated worker threads via `futures::executor::block_on`
/// and are wrapped in `futures::future::Abortable`, so `JoinHandle::abort`
/// stops them at their next suspension point.
#[derive(Clone)]
pub struct FloemExecutor;

impl Executor for FloemExecutor {
    type JoinHandle<T: Send + 'static> = FloemJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();
        let (abort_handle, abort_registration) = futures::future::AbortHandle::new_pair();

        std::thread::spawn(move || {
            if let Ok(result) = futures::executor::block_on(futures::future::Abortable::new(
                future,
                abort_registration,
            )) {
                let _ = tx.send(result);
            }
        });

        FloemJoinHandle {
            abort: abort_handle,
            rx: Some(rx),
            detached: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// A handle to a task spawned by [`FloemExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
pub struct FloemJoinHandle<T> {
    abort: futures::future::AbortHandle,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: std::sync::atomic::AtomicBool,
}

impl<T: Send + 'static> JoinHandle<T> for FloemJoinHandle<T> {
    /// Abort the spawned task.
    ///
    /// Cancellation is cooperative: the wrapped future stops at its next
    /// suspension point. The worker thread then completes without producing
    /// a result.
    fn abort(&self) {
        self.abort.abort();
    }

    /// Detach the task so it runs to completion even if this handle is
    /// dropped. The task's result is discarded.
    fn detach(self) {
        self.detached
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl<T> Drop for FloemJoinHandle<T> {
    fn drop(&mut self) {
        // Structured concurrency: dropping the handle without awaiting or
        // detaching cancels the task.
        if !self.detached.load(std::sync::atomic::Ordering::SeqCst) {
            self.abort.abort();
        }
    }
}

impl<T: Send + 'static> std::future::IntoFuture for FloemJoinHandle<T> {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for FloemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FloemSignal")
            .field("value", &self.inner.get())
            .finish()
    }
}
