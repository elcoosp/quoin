//! Minimal built-in reference implementations for documentation and tests.
//!
//! These types implement the core traits with plain `std` primitives so that
//! doctests and framework-free test suites can exercise quoin APIs without a
//! UI framework. For richer testing support (update counting, deterministic
//! executors, an update log) see the `quoin-testkit` crate.

use crate::effect::DepHandle;
use crate::executor::{Executor, JoinHandle};
use crate::reactive::ReactiveContext;
use crate::signal::Signal;
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};

/// The shared cell behind a [`StaticSignal`]: the value plus the effect
/// dependency handle. Keeping the handle inside the cell means every handle
/// created over the same cell (notably via `use_global`) notifies the same
/// dependency on mutation.
struct StaticCell<T> {
    value: RwLock<T>,
    dep: DepHandle,
}

/// A thread-safe standalone signal backed by `Arc<RwLock<T>>`.
///
/// Mutations notify the core effect runtime (see [`crate::effect`]) but
/// trigger no UI update (there is no UI here), which makes it ideal for
/// doctests that focus on value semantics.
#[derive(Clone)]
pub struct StaticSignal<T> {
    cell: Arc<StaticCell<T>>,
}

impl<T> StaticSignal<T> {
    /// Creates a signal holding `initial`.
    pub fn new(initial: T) -> Self {
        Self {
            cell: Arc::new(StaticCell {
                value: RwLock::new(initial),
                dep: DepHandle::new(),
            }),
        }
    }

    /// Creates a signal sharing an existing cell (used for globals). The
    /// dependency handle travels with the cell, so mutations through any
    /// handle notify the same effect subscribers.
    const fn from_cell(cell: Arc<StaticCell<T>>) -> Self {
        Self { cell }
    }
}

impl<T> std::fmt::Debug for StaticSignal<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticSignal")
            .field("value", &self.cell.value)
            .finish_non_exhaustive()
    }
}

/// Creates a [`StaticSignal`] (convenience for doctests).
pub fn static_signal<T>(initial: T) -> StaticSignal<T> {
    StaticSignal::new(initial)
}

impl<T: Clone + 'static> Signal<T> for StaticSignal<T> {
    fn get(&self) -> T {
        self.cell.dep.track_read();
        self.cell.value.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.cell.dep.track_read();
        f(&self.cell.value.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.cell.value.write().unwrap() = value;
        self.cell.dep.notify();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.cell.value.write().unwrap());
        self.cell.dep.notify();
    }

    fn set_if_eq(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        // Single write-lock acquisition instead of read-then-write.
        let mut guard = self.cell.value.write().unwrap();
        if *guard == value {
            return false;
        }
        *guard = value;
        drop(guard);
        self.cell.dep.notify();
        true
    }
}

impl<T: Clone + 'static> crate::effect::TrackedSignal<T> for StaticSignal<T> {
    fn dep_handle(&self) -> DepHandle {
        self.cell.dep.clone()
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
    const fn ready(result: T) -> Self {
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
///
/// Globals (`provide_global` / `use_global`) live in a thread-local type-map
/// shared by every context on the calling thread, so the full
/// `provide → use → mutate` flow works without a UI framework.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticContext;

impl ReactiveContext for StaticContext {
    type Signal<T: Clone + Send + 'static> = StaticSignal<T>;
    type Executor = StaticExecutor;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        static_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        StaticExecutor
    }

    fn request_update(&self) {
        // No UI to update.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        // Store the value inside a shared cell so every `use_global`
        // consumer observes the same mutations (matching the unified
        // production-adapter semantics). The cell type is the map key, so
        // the shared `globals` store namespaces by context automatically.
        // The effect dependency handle lives inside the cell, so every
        // handle handed out by `use_global` shares one subscription list.
        crate::globals::insert::<Self, Arc<StaticCell<T>>>(Arc::new(StaticCell {
            value: RwLock::new(value),
            dep: DepHandle::new(),
        }));
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        crate::globals::get::<Self, Arc<StaticCell<T>>>().map(StaticSignal::from_cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::ReactiveContextExt;

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
    fn static_signal_set_if_eq_writes_once() {
        let s = static_signal(String::from("same"));
        assert!(!s.set_if_eq(String::from("same")), "equal write skipped");
        assert_eq!(s.get(), "same");
        assert!(s.set_if_eq(String::from("new")), "changed write lands");
        assert_eq!(s.get(), "new");
    }

    #[test]
    fn static_executor_runs_inline() {
        let handle = StaticExecutor.spawn(async { "done".to_string() });
        assert_eq!(handle.get_result(), "done");
    }

    #[test]
    fn static_context_globals_roundtrip() {
        let cx = StaticContext;
        assert!(cx.use_global::<u32>().is_none());
        cx.provide_global(10u32);
        let sig = cx.use_global::<u32>().expect("global provided");
        assert_eq!(sig.get(), 10);
        sig.set(11);
        // A fresh handle observes the mutation (shared store).
        let sig2 = cx.use_global::<u32>().expect("global still present");
        assert_eq!(sig2.get(), 11);
    }

    #[test]
    fn static_context_use_global_or_init_works() {
        let cx = StaticContext;
        let sig = cx.use_global_or_init(|| 7u32);
        assert_eq!(sig.get(), 7);
    }

    #[test]
    fn static_context_globals_are_typed() {
        let cx = StaticContext;
        cx.provide_global("text".to_string());
        assert!(cx.use_global::<u32>().is_none());
        assert_eq!(cx.use_global::<String>().unwrap().get(), "text");
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
