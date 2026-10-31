//! Leptos adapter for quoin — reactive signals, async executors, and clipboard helpers.
//!
//! This crate implements [`ReactiveContext`] for the [Leptos](https://leptos.dev/)
//! web framework, enabling framework-agnostic hooks and state to run inside
//! Leptos components on both WASM (CSR) and server (SSR).
//!
//! # Core Types
//!
//! | Type | Role |
//! |------|------|
//! | [`LeptosContext`] | A zero-sized context. Creates signals via Leptos's `RwSignal`. |
//! | [`LeptosSignal<T>`] | A `Clone` signal backed by `RwSignal<SendWrapper<T>>`. |
//! | [`LeptosExecutor`] | Spawns tasks on a blocking `std::thread` (see notes below). |
//! | [`LeptosJoinHandle<T>`] | A `JoinHandle` wrapping a oneshot channel for result retrieval. |
//!
//! # Creating a Context
//!
//! Call `LeptosContext::new()` inside any Leptos component or effect scope:
//!
//! ```ignore
//! #[component]
//! fn MyComponent() -> impl IntoView {
//!     let ctx = LeptosContext::new();
//!     let count = ctx.create_signal(0u32);
//!     // ...
//! }
//! ```
//!
//! # Signal Threading Model
//!
//! Leptos signals are **not** `Send`. To satisfy quoin's `Signal: Send + Sync` bounds,
//! values are wrapped in [`SendWrapper`](https://docs.rs/send_wrapper). This means:
//!
//! - `LeptosSignal<T>` is `Clone + Send + Sync` and can cross thread boundaries.
//! - The underlying `RwSignal` can only be accessed on the thread that created it
//!   (typically the main/WASM thread). Accessing from another thread will **panic**.
//! - In practice this is fine: Leptos components run on the main thread, and
//!   `LeptosExecutor` spawns work on background threads that communicate results
//!   back via channels (not via signal mutations).
//!
//! # Async Executor
//!
//! `LeptosExecutor` runs futures on dedicated worker threads on native
//! targets (SSR) and on the main thread via `wasm_bindgen_futures` on
//! `wasm32` (CSR). Every spawned future is wrapped in
//! `futures::future::Abortable`, so `JoinHandle::abort` stops it at its
//! next suspension point and dropping the handle cancels it.
//!
//! # Global State (Reactive Owner Tree)
//!
//! `provide_global` wraps the value in `RwSignal<SendWrapper<T>>` and calls
//! Leptos's `provide_context`. `use_global` retrieves it via `use_context`.
//!
//! - Globals are scoped to the current reactive [`Owner`](leptos::prelude::Owner).
//! - A single `provide_global` at the app root propagates to all descendants.
//! - The returned `LeptosSignal` shares the same underlying `RwSignal`, so
//!   mutations are visible to all holders.
//! - Automatic cleanup when the owner is dropped.
//!
//! # Clipboard Helper
//!
//! The [`clipboard_write_text`] function provides a cross-platform (WASM) way to
//! copy text, used internally by the `clipboard_button` element in `quoin_render!`.

use leptos::prelude::*;
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal as QuoinSignal};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Default)]
/// Leptos reactive context.
///
/// Create with `LeptosContext::new()` inside a component.
pub struct LeptosContext;

impl LeptosContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for LeptosContext {
    type Signal<T: Clone + 'static> = LeptosSignal<T>;
    type Executor = LeptosExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        let inner = RwSignal::new(SendWrapper::new(initial));
        LeptosSignal { inner }
    }

    fn executor(&self) -> Self::Executor {
        LeptosExecutor
    }

    fn request_update(&self) {
        // Leptos reactivity is automatic.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        leptos::prelude::provide_context(leptos::prelude::RwSignal::new(SendWrapper::new(value)));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        leptos::prelude::use_context::<leptos::prelude::RwSignal<SendWrapper<T>>>()
            .map(|sig| LeptosSignal { inner: sig })
    }
}

#[derive(Clone)]
/// Leptos-backed reactive signal.
pub struct LeptosSignal<T: Clone + 'static> {
    inner: RwSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> QuoinSignal<T> for LeptosSignal<T> {
    fn get(&self) -> T {
        self.inner.get().take()
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

#[derive(Clone)]
/// Leptos async executor.
///
/// On native targets (SSR) futures run on dedicated worker threads via
/// `futures::executor::block_on`; on `wasm32` they run on the browser's main
/// thread via `wasm_bindgen_futures::spawn_local`. In both cases the future is
/// wrapped in [`futures::future::Abortable`], so [`abort`](JoinHandle::abort)
/// stops it at its next suspension point.
pub struct LeptosExecutor;

impl Executor for LeptosExecutor {
    type JoinHandle<T: Send + 'static> = LeptosJoinHandle<T>;

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

        LeptosJoinHandle {
            abort: abort_handle,
            rx: Some(rx),
            detached: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// A handle to a task spawned by [`LeptosExecutor`].
///
/// Awaiting resolves with the task's result, or `Err(Canceled)` if the task
/// was aborted. Dropping the handle without detaching cancels the task.
pub struct LeptosJoinHandle<T> {
    abort: futures::future::AbortHandle,
    rx: Option<futures::channel::oneshot::Receiver<T>>,
    detached: std::sync::atomic::AtomicBool,
}

impl<T: Send + 'static> JoinHandle<T> for LeptosJoinHandle<T> {
    /// Abort the spawned task.
    ///
    /// Cancellation is cooperative: the wrapped future stops at its next
    /// suspension point. The worker thread (native) or the spawned local task
    /// (wasm) then completes without producing a result.
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

impl<T> Drop for LeptosJoinHandle<T> {
    fn drop(&mut self) {
        // Structured concurrency: dropping the handle without awaiting or
        // detaching cancels the task.
        if !self.detached.load(std::sync::atomic::Ordering::SeqCst) {
            self.abort.abort();
        }
    }
}

impl<T: Send + 'static> std::future::IntoFuture for LeptosJoinHandle<T> {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for LeptosSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeptosSignal")
            .field("value", &self.inner.get())
            .finish()
    }
}

/// Write text to the system clipboard (Web/WASM only).
///
/// Falls back silently if the clipboard API is unavailable (e.g., SSR context).
/// This function is called by the `clipboard_button` element in `quoin_render!`.
pub fn clipboard_write_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            // write_text returns a Promise; we can safely ignore it (fire-and-forget)
            let _ = window.navigator().clipboard().write_text(text);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = text;
    }
}
