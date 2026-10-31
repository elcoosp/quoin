//! Minimal built-in reference implementations for documentation and tests.
//!
//! These types implement the core traits with plain `std` primitives so that
//! doctests and framework-free test suites can exercise quoin APIs without a
//! UI framework. For richer testing support (update counting, deterministic
//! executors, an update log) see the `quoin-testkit` crate.

use crate::executor::{Executor, JoinHandle};
use crate::reactive::ReactiveContext;
use crate::signal::Signal;
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};

/// A thread-safe standalone signal backed by `Arc<RwLock<T>>`.
///
/// Mutations do **not** notify anything (there is no UI to update), which
/// makes it ideal for doctests that focus on value semantics.
#[derive(Clone, Debug)]
pub struct StaticSignal<T> {
    cell: Arc<RwLock<T>>,
}

impl<T> StaticSignal<T> {
    /// Creates a signal holding `initial`.
    pub fn new(initial: T) -> Self {
        Self {
            cell: Arc::new(RwLock::new(initial)),
        }
    }
}

/// Creates a [`StaticSignal`] (convenience for doctests).
pub fn static_signal<T>(initial: T) -> StaticSignal<T> {
    StaticSignal::new(initial)
}

impl<T: Clone + 'static> Signal<T> for StaticSignal<T> {
    fn get(&self) -> T {
        self.cell.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.cell.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.cell.write().unwrap() = value;
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.cell.write().unwrap());
    }
}

/// A standalone executor that runs every future **inline, to completion**,
/// on the calling thread at spawn time.
///
/// Deterministic and allocation-light, but blocking: a future that awaits
/// external events (timers, channels fed by other threads) will hang. The
/// `quoin-testkit` crate provides non-blocking alternatives.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticExecutor;

/// Handle for [`StaticExecutor`]: the future has already finished by the
/// time this handle exists, so `abort` is a no-op and awaiting yields the
/// captured result directly (there is nothing left to cancel).
///
/// The result sits behind a `Mutex` so the handle is `Send + Sync` for any
/// `T: Send`, matching the [`JoinHandle`] supertraits.
pub struct StaticJoinHandle<T> {
    result: Mutex<Option<T>>,
}

impl<T> StaticJoinHandle<T> {
    fn ready(result: T) -> Self {
        Self {
            result: Mutex::new(Some(result)),
        }
    }
}

impl<T> Unpin for StaticJoinHandle<T> {}

impl<T: Send> JoinHandle<T> for StaticJoinHandle<T> {
    fn abort(&self) {
        // The future already completed; nothing to cancel.
    }

    fn detach(self)
    where
        Self: Sized,
    {
        // Already completed; nothing to leak.
    }
}

impl<T> Future for StaticJoinHandle<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(
            self.get_mut()
                .result
                .get_mut()
                .unwrap()
                .take()
                .expect("static join handle polled after completion"),
        )
    }
}

impl Executor for StaticExecutor {
    type JoinHandle<T: Send + 'static> = StaticJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let output = crate::pool::block_on(future);
        StaticJoinHandle::ready(output)
    }
}

/// A zero-sized [`ReactiveContext`] built from [`StaticSignal`] and
/// [`StaticExecutor`], for doctests and framework-free unit tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticContext;

impl ReactiveContext for StaticContext {
    type Signal<T: Clone + 'static> = StaticSignal<T>;
    type Executor = StaticExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        static_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        StaticExecutor
    }

    fn request_update(&self) {
        // No UI to update.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, _value: T) {
        // Intentionally stateless; the testkit provides a full-featured
        // implementation with a shared global store.
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_signal_roundtrip() {
        let s = static_signal(1u32);
        assert_eq!(s.get(), 1);
        s.set(2);
        assert_eq!(s.get(), 2);
        s.update(|v| *v += 1);
        assert_eq!(s.get(), 3);
        let clone = s.clone();
        clone.set(9);
        assert_eq!(s.get(), 9);
    }

    #[test]
    fn static_executor_runs_inline() {
        let handle = StaticExecutor.spawn(async { "done".to_string() });
        assert_eq!(handle.get_result(), "done");
    }

    impl<T> StaticJoinHandle<T> {
        fn get_result(&self) -> T
        where
            T: Clone,
        {
            self.result
                .lock()
                .unwrap()
                .as_ref()
                .cloned()
                .expect("static join handle must hold a result")
        }
    }
}
