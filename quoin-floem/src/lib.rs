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
//! | [`FloemJoinHandle<T>`] | Shared oneshot handle (abort / detach / abort-on-drop) from `quoin-core`. |
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
//! # Global State (Thread-Local Store, Shared Values)
//!
//! `provide_global` and `use_global` use a **thread-local** type-map (same pattern
//! as the GPUI adapter). Globals are only visible on the registering thread.
//!
//! The stored value is the Floem `RwSignal` itself, so every `use_global`
//! consumer shares one reactive cell: mutations propagate between consumers
//! and drive Floem reactivity (unified with the Leptos adapter semantics).
//!
//! The adapter also implements [`quoin_core::LocalExecutor`]: `spawn_local`
//! queues `!Send` futures on the current thread (native), driven by
//! `quoin_core::local::run_local_tasks()`.

use floem_reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use quoin_core::{Executor, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone, Default)]
/// Floem reactive context.
pub struct FloemContext;

impl FloemContext {
    /// Creates a fresh context backed by the shared floem update handle.
    #[must_use]
    pub const fn new() -> Self {
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
        // Store the reactive signal itself so every consumer shares the same
        // cell: mutations propagate between consumers (matching the Leptos
        // adapter; previously each use_global returned an independent copy).
        GLOBAL_STORE.with(|store| {
            store.borrow_mut().insert(
                TypeId::of::<T>(),
                Box::new(RwSignal::new(SendWrapper::new(value))),
            );
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<RwSignal<SendWrapper<T>>>())
                .copied()
                .map(|inner| FloemSignal { inner })
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
        // Shared implementation: the future is wrapped in an `Abortable` and
        // runs on the shared worker pool. See `quoin_core::spawn_abortable`.
        quoin_core::spawn_abortable(future)
    }
}

/// A handle to a task spawned by [`FloemExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
///
/// This is [`quoin_core::AbortJoinHandle`] under a historical name: the
/// handle lifecycle (await / abort / detach / abort-on-drop) is implemented
/// once in `quoin-core` and shared with the Leptos and Dioxus adapters.
pub type FloemJoinHandle<T> = quoin_core::AbortJoinHandle<T>;

impl quoin_core::LocalExecutor for FloemExecutor {
    type JoinHandle<T: 'static> = quoin_core::LocalTask<T>;

    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: std::future::Future + 'static,
        F::Output: 'static,
    {
        // On wasm the task runs on the browser main thread; on native it is
        // queued on the current thread (drive with
        // `quoin_core::local::run_local_tasks()`).
        quoin_core::spawn_local_task(future)
    }
}

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for FloemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FloemSignal")
            .field("value", &self.inner.get())
            .finish()
    }
}
