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
//! | [`DioxusJoinHandle<T>`] | A `JoinHandle` wrapping a oneshot channel. |
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
//! wraps them in `RefCell<Signal<T>>` to satisfy quoin's `Signal` trait, which
//! requires `&self` access (no `&mut self`).
//!
//! - `DioxusSignal<T>` is `Clone` and can be used within a single component tree.
//! - The `RefCell` borrow rules apply: you cannot hold a `.read()` borrow while
//!   calling `.write()`. In practice this means don't nest signal reads/writes
//!   within the same expression.
//! - Signals are **not** thread-safe. Do not send `DioxusSignal` across threads.
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
//! # Global State (Scope-Based)
//!
//! `provide_global` calls Dioxus's `provide_context`. `use_global` calls
//! `try_consume_context`.
//!
//! - Globals are scoped to the Dioxus component scope (`ScopeId`).
//! - `try_consume_context` is used (rather than `consume_context`) to avoid
//!   removing the context on first access, allowing multiple consumers.
//! - Cleaned up when the scope is released.

use dioxus::prelude::*;
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Default)]
/// Dioxus reactive context.
///
/// Use `DioxusContext::new()` inside a Dioxus component.
pub struct DioxusContext;

impl DioxusContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for DioxusContext {
    type Signal<T: Clone + 'static> = DioxusSignal<T>;
    type Executor = DioxusExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        DioxusSignal {
            inner: RefCell::new(Signal::new(initial)),
        }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus reactivity is automatic.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        dioxus::prelude::provide_context(value);
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        dioxus::prelude::try_consume_context::<T>().map(|v| DioxusSignal {
            inner: RefCell::new(Signal::new(v)),
        })
    }
}

#[derive(Clone)]
/// Dioxus-backed reactive signal.
pub struct DioxusSignal<T: Clone + 'static> {
    inner: RefCell<Signal<T>>,
}

impl<T: Clone + 'static> QuoinSignal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.inner.borrow().read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.inner.borrow().read())
    }

    fn set(&self, value: T) {
        *self.inner.borrow_mut().write() = value;
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut *self.inner.borrow_mut().write());
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
        let (tx, rx) = futures::channel::oneshot::channel();
        let (abort_handle, abort_registration) = futures::future::AbortHandle::new_pair();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(result) =
                    futures::future::Abortable::new(future, abort_registration).await
                {
                    let _ = tx.send(result);
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(move || {
                if let Ok(result) = futures::executor::block_on(futures::future::Abortable::new(
                    future,
                    abort_registration,
                )) {
                    let _ = tx.send(result);
                }
            });
        }

        DioxusJoinHandle {
            abort: abort_handle,
            rx: Some(rx),
            detached: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// A handle to a task spawned by [`DioxusExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
pub struct DioxusJoinHandle<T> {
    abort: futures::future::AbortHandle,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: std::sync::atomic::AtomicBool,
}

impl<T: Send + 'static> JoinHandle<T> for DioxusJoinHandle<T> {
    /// Abort the spawned task.
    ///
    /// Cancellation is cooperative: the wrapped future stops at its next
    /// suspension point.
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

impl<T> Drop for DioxusJoinHandle<T> {
    fn drop(&mut self) {
        // Structured concurrency: dropping the handle without awaiting or
        // detaching cancels the task.
        if !self.detached.load(std::sync::atomic::Ordering::SeqCst) {
            self.abort.abort();
        }
    }
}

impl<T: Send + 'static> std::future::IntoFuture for DioxusJoinHandle<T> {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for DioxusSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DioxusSignal")
            .field("value", &self.inner.borrow().read())
            .finish()
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
