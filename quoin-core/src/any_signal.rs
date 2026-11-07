//! Type-erased signals: [`AnySignal<T>`] stores *any* signal of `T`.
//!
//! The second half of ADR-002 ("Hybrid: Trait + optional type-erased
//! wrapper"): the [`Signal<T>`](crate::Signal) trait stays statically
//! dispatched everywhere by default, and `AnySignal<T>` is the opt-in
//! dynamic-dispatch escape hatch for the cases that need to store
//! heterogeneous signal handles in one collection, a struct field of
//! erased context, or a devtools panel.
//!
//! # Cost
//!
//! Erasure dispatches `get`/`set`/`update` through one vtable call, and
//! [`with`](crate::Signal::with) clones the value (the borrow-based read
//! cannot cross the type-erasure boundary without a callback-generic trait
//! method). Hot paths should keep the concrete signal type; erasure is for
//! storage and plumbing, not per-frame reads. Measured overhead lives in
//! `docs/BENCHMARKS.md`.
//!
//! # Example
//!
//! ```
//! use quoin_core::prelude::*;
//!
//! let count = quoin_core::test_support::static_signal(21u32);
//! let erased: AnySignal<u32> = AnySignal::new(count.clone());
//!
//! assert_eq!(erased.get(), 21);
//! erased.set(42);
//! assert_eq!(count.get(), 42, "erasure shares the underlying cell");
//!
//! // Heterogeneous concrete types, one collection:
//! let other: AnySignal<u32> = AnySignal::new(quoin_core::test_support::static_signal(7u32));
//! let signals: Vec<AnySignal<u32>> = vec![erased, other];
//! let total: u32 = signals.iter().map(|s| s.get()).sum();
//! assert_eq!(total, 49);
//! ```

use crate::effect::DepHandle;
use crate::signal::Signal;
use std::fmt;
use std::sync::Arc;

/// The type-erased operation table behind [`AnySignal<T>`].
trait ErasedSignal<T>: Send + Sync {
    fn get_erased(&self) -> T;
    fn set_erased(&self, value: T);
    fn update_erased(&self, f: Box<dyn FnOnce(&mut T) + '_>);
}

/// Stores `S` and forwards the erased operations.
struct Erased<S, T> {
    inner: S,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<S, T> ErasedSignal<T> for Erased<S, T>
where
    S: Signal<T> + Send + Sync + 'static,
    T: Clone + 'static,
{
    fn get_erased(&self) -> T {
        self.inner.get()
    }

    fn set_erased(&self, value: T) {
        self.inner.set(value);
    }

    fn update_erased(&self, f: Box<dyn FnOnce(&mut T) + '_>) {
        self.inner.update(f);
    }
}

/// A dynamically dispatched handle to any signal of `T`.
///
/// See the [module docs](self) for the cost model and ADR-002 context.
pub struct AnySignal<T: Clone + 'static> {
    inner: Arc<dyn ErasedSignal<T>>,
    dep: Option<DepHandle>,
}

impl<T: Clone + 'static> AnySignal<T> {
    /// Erases a concrete signal. The result does not forward an effect
    /// dependency cell; use [`new_tracked`](AnySignal::new_tracked) when the
    /// inner signal is tracked and effect plumbing should see through the
    /// erasure. Reads inside [`create_effect`](crate::effect::create_effect)
    /// track the inner cell either way.
    pub fn new<S>(signal: S) -> Self
    where
        S: Signal<T> + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(Erased {
                inner: signal,
                _marker: std::marker::PhantomData,
            }),
            dep: None,
        }
    }

    /// Erases a [`TrackedSignal`](crate::effect::TrackedSignal), forwarding
    /// its dependency cell.
    pub fn new_tracked<S>(signal: &S) -> Self
    where
        S: crate::effect::TrackedSignal<T> + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(Erased {
                inner: signal.clone(),
                _marker: std::marker::PhantomData,
            }),
            dep: Some(signal.dep_handle()),
        }
    }

    /// The inner signal's dependency cell, when erased through
    /// [`new_tracked`](AnySignal::new_tracked) — lets effect-aware plumbing
    /// see through the erasure.
    #[must_use]
    pub fn dep_handle(&self) -> Option<DepHandle> {
        self.dep.clone()
    }

    /// `true` when both handles erase the same underlying signal cell.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: Clone + 'static> Clone for AnySignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            dep: self.dep.clone(),
        }
    }
}

impl<T: Clone + fmt::Debug + 'static> fmt::Debug for AnySignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnySignal")
            .field("value", &self.inner.get_erased())
            .finish_non_exhaustive()
    }
}

impl<T: Clone + 'static> Signal<T> for AnySignal<T> {
    fn get(&self) -> T {
        self.inner.get_erased()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        // Borrow-based reads cannot cross the erasure boundary; lend the
        // cloned value from this stack frame instead (documented cost).
        let value = self.inner.get_erased();
        f(&value)
    }

    fn set(&self, value: T) {
        self.inner.set_erased(value);
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        // Route through the erased table so the inner signal's own `update`
        // semantics (single lock, one notification, one update count) apply.
        self.inner.update_erased(Box::new(f));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{TrackedSignal, create_effect};
    use crate::test_support::static_signal;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn any_signal_round_trips() {
        let source = static_signal(21u32);
        let erased: AnySignal<u32> = AnySignal::new(source.clone());
        assert_eq!(erased.get(), 21);
        erased.set(42);
        assert_eq!(source.get(), 42, "erasure shares the underlying cell");
        assert_eq!(erased.get(), 42);
    }

    #[test]
    fn any_signal_with_lends_value() {
        let source = static_signal(vec![1u8, 2, 3]);
        let erased: AnySignal<Vec<u8>> = AnySignal::new(source);
        let len = erased.with(std::vec::Vec::len);
        assert_eq!(len, 3);
    }

    #[test]
    fn any_signal_update_mutates_shared_cell() {
        let source = static_signal(10u32);
        let erased: AnySignal<u32> = AnySignal::new(source.clone());
        erased.update(|v| *v += 5);
        assert_eq!(source.get(), 15);
        assert_eq!(erased.get(), 15);
    }

    #[test]
    fn any_signal_stores_heterogeneous_sources() {
        let a = static_signal(1u32);
        let b = crate::NotifierSignal::new(2u32, || {});
        let signals: Vec<AnySignal<u32>> = vec![AnySignal::new(a), AnySignal::new(b)];
        let total: u32 = signals.iter().map(crate::Signal::get).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn any_signal_clones_share_cell_and_identity() {
        let source = static_signal(0u32);
        let erased: AnySignal<u32> = AnySignal::new(source.clone());
        let clone = erased.clone();
        clone.set(9);
        assert_eq!(erased.get(), 9);
        assert!(erased.ptr_eq(&clone), "clones erase the same cell");

        let other: AnySignal<u32> = AnySignal::new(static_signal(9u32));
        assert!(!erased.ptr_eq(&other));
    }

    #[test]
    fn any_signal_debug_shows_value() {
        let erased: AnySignal<u32> = AnySignal::new(static_signal(41u32));
        assert_eq!(format!("{erased:?}"), "AnySignal { value: 41, .. }");
    }

    #[test]
    fn any_signal_exposes_tracked_dependency() {
        let source = static_signal(0u32);
        let erased: AnySignal<u32> = AnySignal::new_tracked(&source);
        let erased_dep = erased.dep_handle().expect("tracked erasure → Some");
        assert!(
            erased_dep.ptr_eq(&source.dep_handle()),
            "erasure forwards the inner cell"
        );
        let untracked: AnySignal<u32> = AnySignal::new(static_signal(0u32));
        assert!(untracked.dep_handle().is_none(), "plain erasure → None");
    }

    #[test]
    fn any_signal_effect_reads_are_tracked() {
        let source = static_signal(1u32);
        let erased: AnySignal<u32> = AnySignal::new(source.clone());
        let count = Arc::new(AtomicUsize::new(0));
        let (e2, c2) = (erased.clone(), count.clone());
        let _handle = create_effect(move || {
            e2.get();
            c2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);
        source.set(2);
        assert_eq!(count.load(Ordering::SeqCst), 2, "tracked through erasure");
    }
}
