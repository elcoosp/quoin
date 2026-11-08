//! Dioxus adapter for quoin — reactive signals and async executors.
//!
//! This crate implements [`ReactiveContext`] for the [Dioxus](https://dioxuslabs.com/)
//! framework, enabling framework-agnostic hooks and state to run inside
//! Dioxus components.
//!
//! # Core Types
//!
//! | Type | Role |
//!------|------|
//! | [`DioxusContext`] | A zero-sized context. Creates signals via Dioxus's `Signal`. |
//! | [`DioxusSignal<T>`] | A `Clone` signal backed by `RefCell<Signal<T>>`. |
//! | [`DioxusExecutor`] | Spawns tasks on a blocking `std::thread`. |
//! | [`DioxusJoinHandle<T>`] | Shared oneshot handle (abort / detach / abort-on-drop) from `quoin-core`. |
//!
//! # Creating a Context
//!
//! Call `DioxusContext::new()` inside a Dioxus component, typically via `use_hook`:
//!
//! ```ignore
//! fn app() -> Element {
//!     let ctx = use_hook(DioxusContext::new);
//!     let count = ctx.create_signal(0u32);
//!     // ...
//! }
//! ```
//!
//! # Signal Threading Model
//!
//! Dioxus signals are `Copy` but tied to the Dioxus reactive graph. This adapter
//! wraps them in `RefCell<SendWrapper<Signal<T>>>` to satisfy quoin's `Signal`
//! trait, which requires `&self` access (no `&mut self`).
//!
//! - `DioxusSignal<T>` is `Clone` and can be used within a single component tree.
//! - The `RefCell` borrow rules apply: you cannot hold a `.read()` borrow while
//!   calling `.write()`. In practice this means don't nest signal reads/writes
//!   within the same expression.
//! - Signals must only be **accessed** on the thread that created them: the
//!   underlying Dioxus `Signal` sits in a `SendWrapper`, which makes the handle
//!   `Send` (core effect closures capture it and re-run on the notifying
//!   thread) while any cross-thread dereference panics — the same threading
//!   model the Leptos adapter documents. The handle stays `!Sync`.
//!
//! # Async Executor
//!
//! `DioxusExecutor` runs futures on dedicated worker threads on native
//! targets (desktop) and on the main thread via `wasm_bindgen_futures` on
//! `wasm32` (web). Every spawned future is wrapped in
//! `futures::future::Abortable`, so `JoinHandle::abort` stops it at its
//! next suspension point and dropping the handle cancels it. For UI-bound
//! work that must mutate signals directly, prefer spawning through Dioxus's
//! own `spawn` inside components.
//!
//! # Global State (Scope-Based, Shared)
//!
//! `provide_global` provides a Dioxus `Signal<T>` (not the raw value);
//! `use_global` consumes it via `try_consume_context` and wraps it in a
//! `DioxusSignal`. Consequences:
//!
//! - Globals are scoped to the Dioxus component scope (`ScopeId`).
//! - **Shared semantics**: mutations through any consumer's signal are
//!   visible to all consumers and trigger Dioxus re-renders — matching the
//!   Leptos adapter (and the rest of the workspace since the globals were
//!   unified).
//! - `try_consume_context` is used (rather than `consume_context`) to avoid
//!   removing the context on first access, allowing multiple consumers.
//! - Cleaned up when the scope is released.
//!
//! The adapter also implements [`quoin_core::LocalExecutor`]: `spawn_local`
//! runs `!Send` futures on the browser main thread (wasm) or queues them on
//! the current thread (native).

use dioxus::prelude::*;
use quoin_core::{DepHandle, Executor, ReactiveContext, Signal as QuoinSignal, TrackedSignal};
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;

#[derive(Clone, Default)]
/// Dioxus reactive context.
///
/// Use `DioxusContext::new()` inside a Dioxus component.
pub struct DioxusContext;

impl DioxusContext {
    /// Creates a fresh context. Call inside a Dioxus component so context
    /// and scope lookups resolve against the current reactive scope.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ReactiveContext for DioxusContext {
    type Signal<T: Clone + Send + 'static> = DioxusSignal<T>;
    type Executor = DioxusExecutor;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        DioxusSignal {
            inner: RefCell::new(SendWrapper::new(Signal::new(initial))),
            dep: DepHandle::new(),
        }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus reactivity is automatic.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        // Provide a Dioxus signal (not the raw value) so every consumer
        // shares the same reactive cell: mutations through one consumer's
        // `use_global` signal are visible to all of them. This matches the
        // Leptos adapter's semantics (globals are shared, never copied).
        //
        // The shared core-effect dependency cell rides alongside as a second
        // context, so every `use_global` consumer observes core-effect
        // notifications from writes through any other consumer's handle.
        dioxus::prelude::provide_context(GlobalDep::<T> {
            dep: DepHandle::new(),
            _marker: PhantomData,
        });
        dioxus::prelude::provide_context(dioxus::prelude::Signal::new(value));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        dioxus::prelude::try_consume_context::<dioxus::prelude::Signal<T>>().map(|sig| {
            let dep = dioxus::prelude::try_consume_context::<GlobalDep<T>>()
                .map(|shared| shared.dep)
                .unwrap_or_default();
            DioxusSignal {
                inner: RefCell::new(SendWrapper::new(sig)),
                dep,
            }
        })
    }
}

/// The shared dependency cell behind `provide_global` (see
/// [`ReactiveContext::provide_global`]): keyed by `T` through dioxus's
/// type-keyed context map, so every `use_global::<T>` handle observes the
/// same core-effect graph node.
#[derive(Clone)]
struct GlobalDep<T> {
    dep: DepHandle,
    _marker: PhantomData<fn() -> T>,
}

#[derive(Clone)]
/// Dioxus-backed reactive signal.
///
/// The underlying Dioxus `Signal` lives in a [`SendWrapper`]: the handle is
/// `Send` (core effect closures capture it and re-run on the notifying
/// thread) while every cross-thread dereference panics — access stays on the
/// creating thread, the same model the Leptos adapter documents.
pub struct DioxusSignal<T: Clone + 'static> {
    inner: RefCell<SendWrapper<Signal<T>>>,
    /// Core effect-graph cell: reads track it, writes notify it (see
    /// [`TrackedSignal`]); `derive(Clone)` shares the cell across handles.
    dep: DepHandle,
}

impl<T: Clone + 'static> QuoinSignal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.dep.track_read();
        self.inner.borrow().read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.dep.track_read();
        f(&self.inner.borrow().read())
    }

    fn set(&self, value: T) {
        *self.inner.borrow_mut().write() = value;
        self.dep.notify();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.borrow_mut().write());
        self.dep.notify();
    }
}

impl<T: Clone + 'static> TrackedSignal<T> for DioxusSignal<T> {
    fn dep_handle(&self) -> DepHandle {
        self.dep.clone()
    }
}

/// Dioxus async executor.
///
/// On native targets futures run on dedicated worker threads via
/// `futures::executor::block_on`; on `wasm32` they run on the browser's main
/// thread via `wasm_bindgen_futures::spawn_local`. In both cases the future is
/// wrapped in `futures::future::Abortable`, so `JoinHandle::abort` stops it at
/// its next suspension point.
#[derive(Clone)]
pub struct DioxusExecutor;

impl Executor for DioxusExecutor {
    type JoinHandle<T: Send + 'static> = DioxusJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // Shared implementation: the future is wrapped in an `Abortable` and
        // runs on the shared worker pool (native) or the browser main thread
        // (wasm). See `quoin_core::spawn_abortable`.
        quoin_core::spawn_abortable(future)
    }
}

/// A handle to a task spawned by [`DioxusExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
///
/// This is [`quoin_core::AbortJoinHandle`] under a historical name: the
/// handle lifecycle (await / abort / detach / abort-on-drop) is implemented
/// once in `quoin-core` and shared with the Leptos and Floem adapters.
pub type DioxusJoinHandle<T> = quoin_core::AbortJoinHandle<T>;

impl quoin_core::LocalExecutor for DioxusExecutor {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for DioxusSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DioxusSignal")
            .field("value", &self.inner.borrow().read())
            // The dep cell is shared bookkeeping, not part of the value.
            .finish_non_exhaustive()
    }
}

impl<T: Clone + 'static> DioxusSignal<T> {
    /// Read the current value. Preferred over `QuoinSignal::get()` in Dioxus
    /// code because it avoids Dioxus 0.7 ReadableVecExt/ReadableHashMapExt
    /// blanket impl trait resolution conflicts.
    pub fn get(&self) -> T {
        QuoinSignal::get(self)
    }

    /// Access the value through a closure without cloning.
    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        QuoinSignal::with(self, f)
    }

    /// Set the value.
    pub fn set(&self, value: T) {
        QuoinSignal::set(self, value);
    }

    /// Update the value with a closure.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        QuoinSignal::update(self, f);
    }
}
