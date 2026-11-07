//! Shared `Arc<RwLock<T>>` signal with an update notifier.
//!
//! The pull-based adapters (GPUI, Xilem) implement their reactive signal as
//! a plain `Arc<RwLock<T>>` cell plus an "ask the framework to re-render"
//! notifier invoked on every mutation (suppressed inside a
//! [`batch`](crate::batch)). That implementation used to be hand-copied in
//! both adapter crates; it now lives here once and both adapters expose
//! their historical signal type as an alias over [`NotifierSignal`].
//!
//! The notifier is invoked *after* the mutation is published and skipped
//! entirely while a batch is active — the batch itself issues a single
//! update request when it ends.

use crate::batch;
use crate::effect::DepHandle;
use crate::signal::Signal;
use std::sync::{Arc, RwLock};

/// A reactive value cell that calls `notify` after every mutation.
///
/// `notify` receives no arguments: adapters capture whatever context handle
/// they need (a shared notifier slot, a framework context) when building the
/// signal. The closure must be `Send + Sync` so the signal can cross threads
/// like every other quoin signal.
///
/// Every mutation also notifies the signal's effect dependency cell (see
/// [`crate::effect`]), so core effects re-run on GPUI/Xilem-style signals
/// without adapter support.
#[derive(Clone)]
pub struct NotifierSignal<T: Clone + 'static> {
    inner: Arc<RwLock<T>>,
    notify: Arc<dyn Fn() + Send + Sync>,
    dep: DepHandle,
}

impl<T: Clone + 'static> NotifierSignal<T> {
    /// Creates a signal holding `initial`; `notify` runs after each `set` /
    /// `update` that is not suppressed by an active batch.
    pub fn new(initial: T, notify: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(RwLock::new(initial)),
            notify: Arc::new(notify),
            dep: DepHandle::new(),
        }
    }
}

impl<T: Clone + 'static> NotifierSignal<T> {
    /// Invokes the notifier unless a batch suppresses it.
    fn notify_after_mutation(&self) {
        if batch::should_notify() {
            (self.notify)();
        }
    }
}

impl<T: Clone + 'static> Signal<T> for NotifierSignal<T> {
    fn get(&self) -> T {
        self.dep.track_read();
        self.inner.read().unwrap().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.dep.track_read();
        f(&self.inner.read().unwrap())
    }

    fn set(&self, value: T) {
        *self.inner.write().unwrap() = value;
        self.dep.notify();
        self.notify_after_mutation();
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        // Release the lock before notifying: the notifier may mutate other
        // signals of the same context from the same thread.
        let mut guard = self.inner.write().unwrap();
        f(&mut guard);
        drop(guard);
        self.dep.notify();
        self.notify_after_mutation();
    }

    fn set_if_eq(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        // Single write-lock acquisition: compare and write under one guard
        // instead of the default read-then-write (two acquisitions).
        let mut guard = self.inner.write().unwrap();
        if *guard == value {
            return false;
        }
        *guard = value;
        drop(guard);
        self.dep.notify();
        self.notify_after_mutation();
        true
    }
}

impl<T: Clone + 'static> crate::effect::TrackedSignal<T> for NotifierSignal<T> {
    fn dep_handle(&self) -> DepHandle {
        self.dep.clone()
    }
}

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for NotifierSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The notifier is context-shared bookkeeping, not part of the value;
        // expose it as non-exhaustive instead of a field.
        f.debug_struct("NotifierSignal")
            .field(
                "value",
                &self
                    .inner
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::ReactiveContextExt as _;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn set_and_get_round_trip() {
        let signal = NotifierSignal::new(1u32, || {});
        signal.set(2);
        assert_eq!(signal.get(), 2);
    }

    #[test]
    fn notifier_runs_on_set_and_update() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_signal = Arc::clone(&count);
        let signal = NotifierSignal::new(0u32, move || {
            count_for_signal.fetch_add(1, Ordering::SeqCst);
        });
        signal.set(1);
        signal.update(|v| *v += 1);
        assert_eq!(signal.get(), 2);
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    /// A `StaticContext` wrapper that counts `request_update` calls, so the
    /// batch contract ("N mutations, one update request") is observable.
    #[derive(Clone)]
    struct CountingContext {
        updates: Arc<AtomicUsize>,
    }

    impl crate::reactive::ReactiveContext for CountingContext {
        type Signal<T: Clone + Send + 'static> = crate::test_support::StaticSignal<T>;
        type Executor = crate::test_support::StaticExecutor;

        fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
            crate::test_support::StaticSignal::new(initial)
        }

        fn executor(&self) -> Self::Executor {
            crate::test_support::StaticExecutor
        }

        fn request_update(&self) {
            self.updates.fetch_add(1, Ordering::SeqCst);
        }

        fn provide_global<T: Clone + Send + Sync + 'static>(&self, _value: T) {
            // Not exercised by these tests.
        }

        fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
            None
        }
    }

    #[test]
    fn batch_suppresses_notifications_until_it_ends() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_signal = Arc::clone(&count);
        let signal = NotifierSignal::new(0u32, move || {
            count_for_signal.fetch_add(1, Ordering::SeqCst);
        });
        let cx = CountingContext {
            updates: Arc::new(AtomicUsize::new(0)),
        };
        let issued = cx.batch(|| {
            signal.set(1);
            signal.update(|v| *v += 10);
            assert_eq!(
                count.load(Ordering::SeqCst),
                0,
                "per-write notifier suppressed inside batch"
            );
            assert_eq!(
                cx.updates.load(Ordering::SeqCst),
                0,
                "no update requested while batching"
            );
            signal.get()
        });
        assert_eq!(issued, 11);
        assert_eq!(
            count.load(Ordering::SeqCst),
            0,
            "signal notifier never fires for batched writes"
        );
        assert_eq!(
            cx.updates.load(Ordering::SeqCst),
            1,
            "the batch issues exactly one update request"
        );
        // Outside a batch the notifier fires again.
        signal.set(12);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn set_if_eq_skips_equal_writes_and_notifications() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_signal = Arc::clone(&count);
        let signal = NotifierSignal::new("a".to_string(), move || {
            count_for_signal.fetch_add(1, Ordering::SeqCst);
        });
        assert!(!signal.set_if_eq("a".to_string()));
        assert_eq!(count.load(Ordering::SeqCst), 0, "equal write skipped");
        assert!(signal.set_if_eq("b".to_string()));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!(signal.get(), "b");
    }

    #[test]
    fn clones_share_the_cell_and_notifier() {
        let notified = Arc::new(AtomicUsize::new(0));
        let notified_for_signal = Arc::clone(&notified);
        let signal = NotifierSignal::new(0u8, move || {
            notified_for_signal.fetch_add(1, Ordering::SeqCst);
        });
        let clone = signal.clone();
        clone.set(1);
        assert_eq!(signal.get(), 1, "clone shares the value cell");
        assert_eq!(notified.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn mutation_visible_through_with_borrow() {
        let signal = NotifierSignal::new(vec![1u32, 2], || {});
        let sum = signal.with(|values| values.iter().sum::<u32>());
        assert_eq!(sum, 3);
    }

    #[test]
    fn debug_shows_value_and_hides_notifier() {
        let signal = NotifierSignal::new(41i64, || {});
        let rendered = format!("{signal:?}");
        assert!(rendered.contains("41"), "value must be visible: {rendered}");
        assert!(
            rendered.contains(", .."),
            "notifier hidden as non-exhaustive: {rendered}"
        );
    }
}
