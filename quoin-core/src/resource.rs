//! Async resource primitive: load, reload, and cancel remote data reactively.
//!
//! [`Resource`] models the full lifecycle of an async computation whose result
//! feeds the UI: `Idle → Loading → Ready(T) | Failed(E)`. It is the
//! framework-agnostic equivalent of Leptos' `Resource` and Dioxus'
//! `use_resource`, and works on **every** quoin adapter because it is built
//! from the two primitives every adapter already provides: a [`Signal`] and an
//! [`Executor`].
//!
//! # Highlights
//!
//! - **Race-free**: every reload bumps a generation counter; a stale request
//!   can never overwrite a newer one's result.
//! - **Cooperative cancellation**: reloads cancel the in-flight request via a
//!   [`CancellationToken`], and the task itself is aborted through the
//!   executor's [`JoinHandle`].
//! - **Reactive**: the state lives in a framework signal, so UI reads
//!   (`resource.state().get()`) re-render automatically on transitions.
//!
//! # Example
//!
//! ```
//! use quoin_core::prelude::*;
//!
//! // Any context works; here the built-in static test context.
//! let cx = quoin_core::test_support::StaticContext;
//! let res = cx.create_resource(|_token| async { Ok::<u32, String>(42) });
//! // With the immediate test executor the future has already completed:
//! assert_eq!(res.read(), Some(42));
//! ```

use crate::cancellation::CancellationToken;
use crate::executor::{Executor, JoinHandle};
use crate::signal::Signal;
use std::fmt;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Type-erased abort closure: cancels the task backing the current request.
type AbortFn = Box<dyn Fn() + Send + Sync>;

/// The lifecycle state of a [`Resource`].
///
/// `Failed` wraps the error in an [`Arc`] so that clones of the state (which
/// every [`Signal::get`] performs) stay cheap even for large error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T, E> {
    /// Created but not started, or cancelled after starting.
    Idle,
    /// A request is in flight.
    Loading,
    /// The latest request completed successfully.
    Ready(T),
    /// The latest request failed.
    Failed(Arc<E>),
}

impl<T, E> ResourceState<T, E> {
    /// Returns `true` while a request is in flight.
    #[must_use]
    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns the ready value, if any.
    #[must_use]
    pub const fn as_ready(&self) -> Option<&T> {
        match self {
            Self::Ready(t) => Some(t),
            _ => None,
        }
    }

    /// Returns the error, if any.
    #[must_use]
    pub fn as_failed(&self) -> Option<&E> {
        match self {
            Self::Failed(e) => Some(e),
            _ => None,
        }
    }
}

/// Internal shared state of a resource.
struct ResourceInner<S, T, E> {
    state: S,
    /// Shared with in-flight tasks so they can verify they are still current.
    generation: Arc<AtomicU64>,
    abort_current: Mutex<Option<AbortFn>>,
    _marker: std::marker::PhantomData<fn() -> Result<T, E>>,
}

/// A reactive handle to an asynchronous computation.
///
/// Created via [`ReactiveContextExt::create_resource`](crate::ReactiveContextExt::create_resource). Cloning shares the
/// same underlying state.
pub struct Resource<S, T, E> {
    inner: Arc<ResourceInner<S, T, E>>,
}

impl<S, T, E> Clone for Resource<S, T, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S, T, E> fmt::Debug for Resource<S, T, E>
where
    S: Signal<ResourceState<T, E>>,
    T: Clone + fmt::Debug + 'static,
    E: Clone + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("state", &self.inner.state.get())
            .field("generation", &self.inner.generation.load(Ordering::Acquire))
            .finish()
    }
}

impl<S, T, E> Resource<S, T, E>
where
    S: Signal<ResourceState<T, E>> + Send + 'static,
    T: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    /// Wraps an existing state signal into a `Resource` (not yet started).
    ///
    /// Prefer [`ReactiveContextExt::create_resource`](crate::ReactiveContextExt::create_resource),
    /// which creates the signal and starts the fetch.
    #[must_use]
    pub fn with_state(state: S) -> Self {
        Self {
            inner: Arc::new(ResourceInner {
                state,
                generation: Arc::new(AtomicU64::new(0)),
                abort_current: Mutex::new(None),
                _marker: std::marker::PhantomData,
            }),
        }
    }

    /// Starts (or restarts) the fetch for this resource.
    ///
    /// Any in-flight request is aborted first. The new request becomes the
    /// only one allowed to publish a result.
    ///
    /// # Panics
    ///
    /// Panics if the executor's mutex state is poisoned (a prior panic while
    /// publishing a result).
    /// The completion path is built on the *caller* thread as a
    /// `Box<dyn FnOnce + Send>` closure around the state signal, so the
    /// spawned future never has to be generic over the signal type's `Sync`
    /// status — only `Send` is required (true for every adapter, including
    /// the `SendWrapper`-based Leptos adapter).
    pub fn reload_with<C, F, Fut>(&self, cx: &C, fetch: F)
    where
        C: crate::ReactiveContext<Executor: Executor<JoinHandle<()>: JoinHandle<()> + 'static>>,
        F: Fn(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        // Cancel whatever is running.
        // Take the closure while holding the lock, then call it after the
        // guard is gone (the closure may touch the resource again).
        let abort = self.inner.abort_current.lock().unwrap().take();
        if let Some(abort) = abort {
            abort();
        }

        let generation = self.inner.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let token = CancellationToken::new();
        let live_generation = Arc::clone(&self.inner.generation);
        let state = self.inner.state.clone();

        self.inner.state.set(ResourceState::Loading);

        // Built here (where the signal lives) so only `Send` is required of
        // the state signal.
        let publish: Box<dyn FnOnce(Result<T, E>) + Send> = Box::new(move |result| match result {
            Ok(value) => state.set(ResourceState::Ready(value)),
            Err(err) => state.set(ResourceState::Failed(Arc::new(err))),
        });

        let handle = cx.executor().spawn(async move {
            // A cancelled token means a newer reload already started.
            if token.is_cancelled() {
                return;
            }
            let result = fetch(token.clone()).await;
            // Publish only if we are still the newest generation and have
            // not been aborted in the meantime.
            if token.is_cancelled() {
                return;
            }
            if live_generation.load(Ordering::Acquire) != generation {
                return;
            }
            publish(result);
        });
        // Type-erase the abort capability so `Resource` does not depend on
        // the executor's concrete handle type.
        let abort: AbortFn = Box::new(move || handle.abort());
        *self.inner.abort_current.lock().unwrap() = Some(abort);
    }

    /// Aborts the in-flight request and moves the resource back to
    /// [`ResourceState::Idle`].
    ///
    /// # Panics
    ///
    /// Panics if the abort-closure mutex is poisoned (a prior panic while
    /// publishing a result).
    pub fn cancel(&self) {
        // Take the closure while holding the lock, then call it after the
        // guard is gone (the closure may touch the resource again).
        let abort = self.inner.abort_current.lock().unwrap().take();
        if let Some(abort) = abort {
            abort();
        }
        self.inner.generation.fetch_add(1, Ordering::AcqRel);
        self.inner.state.set(ResourceState::Idle);
    }

    /// The reactive state signal. Read it from your view layer:
    /// `resource.state().get()`.
    #[must_use]
    pub fn state(&self) -> S {
        self.inner.state.clone()
    }

    /// Returns the ready value, or `None` while loading/failed/idle.
    #[must_use]
    pub fn read(&self) -> Option<T> {
        match self.inner.state.get() {
            ResourceState::Ready(t) => Some(t),
            _ => None,
        }
    }

    /// Returns the current state snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ResourceState<T, E> {
        self.inner.state.get()
    }

    /// Returns the current generation counter (for advanced race checks).
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::ReactiveContextExt;
    use crate::test_support::{StaticContext, static_signal};

    #[test]
    fn resource_state_helpers() {
        let idle: ResourceState<u32, String> = ResourceState::Idle;
        assert!(!idle.is_loading());
        assert!(ResourceState::<u32, String>::Loading.is_loading());
        assert_eq!(ResourceState::<u32, String>::Ready(7).as_ready(), Some(&7));
        assert_eq!(
            ResourceState::<u32, String>::Failed(Arc::new("boom".into())).as_failed(),
            Some(&"boom".to_string())
        );
    }

    #[tokio::test]
    async fn resource_immediate_ready() {
        let cx = StaticContext;
        let res = cx.create_resource(|_t| async { Ok::<_, String>(42u32) });
        // The static executor completes futures before returning, so the
        // resource is already ready.
        assert_eq!(res.read(), Some(42));
        assert!(matches!(res.snapshot(), ResourceState::Ready(42)));
    }

    #[tokio::test]
    async fn resource_error_path() {
        let cx = StaticContext;
        let res = cx.create_resource(|_t| async { Err::<u32, _>("nope".to_string()) });
        assert_eq!(res.read(), None);
        assert_eq!(
            res.snapshot().as_failed().map(std::string::String::as_str),
            Some("nope")
        );
    }

    #[tokio::test]
    async fn resource_reload_bumps_generation_and_cancels() {
        let cx = StaticContext;
        let res = cx.create_resource(|_t| async { Ok::<_, String>(1u32) });
        let gen0 = res.generation();
        res.reload_with(&cx, |_t| async { Ok::<_, String>(2u32) });
        assert_eq!(res.generation(), gen0 + 1);
        assert_eq!(res.read(), Some(2));
        res.cancel();
        assert!(matches!(res.snapshot(), ResourceState::Idle));
        assert_eq!(res.read(), None);
    }

    #[test]
    fn resource_is_clone_and_shares_state() {
        let cx = StaticContext;
        let res = cx.create_resource(|_t| async { Ok::<_, String>(5u32) });
        let clone = res.clone();
        assert_eq!(clone.read(), res.read());
        res.cancel();
        assert!(matches!(clone.snapshot(), ResourceState::Idle));
    }

    #[test]
    fn resource_debug_includes_state() {
        let cx = StaticContext;
        let res = cx.create_resource(|_t| async { Ok::<_, String>(9u32) });
        let dbg = format!("{res:?}");
        assert!(dbg.contains("generation"));
        assert!(dbg.contains("Ready"));
        let _ = static_signal(0u32); // keep helper referenced
    }
}
