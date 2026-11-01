//! Derived (computed) signals that transform another signal's value.
//!
//! A [`Derived`] signal wraps an existing [`Signal`] and computes its value
//! lazily from the source signal on every read. This mirrors "computed" or
//! "memo" primitives found in fine-grained reactive frameworks, but works
//! uniformly across **all** quoin adapters:
//!
//! - On fine-grained frameworks (Leptos, Dioxus, Floem) the source read inside
//!   the derive closure is tracked by the reactive graph, so any consumer that
//!   reads the derived value inside a reactive scope re-runs when the source
//!   changes.
//! - On pull-based frameworks (GPUI, Xilem) a source mutation triggers a
//!   re-render via the context notifier, and the next read recomputes.
//!
//! # Example
//!
//! ```
//! use quoin_core::prelude::*;
//!
//! let source = quoin_core::test_support::static_signal(3u32);
//! let doubled = quoin_core::derived(&source, |v| v * 2);
//! assert_eq!(doubled.get(), 6);
//! ```

use crate::signal::Signal;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Panics with the unified read-only signal message.
fn read_only_panic(what: &str) -> ! {
    panic!(
        "{what} is read-only; mutate the source signal instead. \
         If you need a writable projection, create two signals and keep them \
         in sync from the writer side."
    );
}

/// A signal whose value is computed from another signal.
///
/// See the [module docs](self) for details. `Derived` implements
/// [`Signal<U>`], so it can be stored, cloned, and passed to any API that
/// expects a signal — including other `Derived` values (chaining).
pub struct Derived<S, T, U, F> {
    source: S,
    f: F,
    _marker: std::marker::PhantomData<fn(&T) -> U>,
}

impl<S, T, U, F> Clone for Derived<S, T, U, F>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            f: self.f.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, T, U, F> fmt::Debug for Derived<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + fmt::Debug + 'static,
    U: fmt::Debug,
    F: Fn(&T) -> U,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Derived")
            .field("value", &(self.f)(&self.source.get()))
            .finish()
    }
}

/// Creates a derived signal that computes `U` from the source signal's `T`.
///
/// The closure runs on every read of the derived signal. Keep it cheap and
/// side-effect free.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let names = quoin_core::test_support::static_signal(vec![
///     "alice".to_string(),
///     "bob".to_string(),
/// ]);
/// let count = quoin_core::derived(&names, |names: &Vec<String>| names.len());
/// assert_eq!(count.get(), 2);
/// ```
pub fn derived<S, T, U, F>(source: &S, f: F) -> Derived<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    Derived {
        source: source.clone(),
        f,
        _marker: std::marker::PhantomData,
    }
}

impl<S, T, U, F> Signal<U> for Derived<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    fn get(&self) -> U {
        (self.f)(&self.source.get())
    }

    fn with<V>(&self, f: impl FnOnce(&U) -> V) -> V {
        // Compute once, lend the result to the closure. The temporary lives
        // on the stack for the duration of `f`, which is safe because `U` is
        // owned by this frame.
        let value = self.get();
        f(&value)
    }

    fn set(&self, _value: U) {
        // A derived signal is read-only by construction: its value is fully
        // determined by the source. Mutating it would be ambiguous (which
        // source value produced it?). Panicking with a clear message beats a
        // silent no-op.
        read_only_panic("quoin_core::Derived");
    }

    fn update(&self, _f: impl FnOnce(&mut U)) {
        read_only_panic("quoin_core::Derived");
    }
}

impl<S, T, U, F> Derived<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    /// Returns a reference to the underlying source signal.
    pub const fn source(&self) -> &S {
        &self.source
    }

    /// Chains another transformation, producing a second derived signal.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::prelude::*;
    ///
    /// let n = quoin_core::test_support::static_signal(2i32);
    /// let x = quoin_core::derived(&n, |v| v * 10).map(|v| v + 1);
    /// assert_eq!(x.get(), 21);
    /// ```
    pub fn map<V, G>(self, g: G) -> Derived<Self, U, V, G>
    where
        V: Clone + 'static,
        G: Fn(&U) -> V + Clone + 'static,
    {
        derived(&self, g)
    }
}

/// Combines two signals into a read-only signal of their tuple.
///
/// The combined value is recomputed on each read from both sources. On
/// fine-grained frameworks both source reads are tracked, so consumers
/// re-run when either input changes.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let a = quoin_core::test_support::static_signal(1u32);
/// let b = quoin_core::test_support::static_signal("x".to_string());
/// let both = quoin_core::zip(&a, &b);
/// assert_eq!(both.get(), (1u32, "x".to_string()));
/// ```
pub fn zip<A, B, TA, TB>(a: &A, b: &B) -> Zipped<A, B, TA, TB>
where
    A: Signal<TA>,
    B: Signal<TB>,
    TA: Clone + 'static,
    TB: Clone + 'static,
{
    Zipped {
        a: a.clone(),
        b: b.clone(),
        _marker: std::marker::PhantomData,
    }
}

/// A read-only signal combining two source signals into a tuple.
///
/// Created by [`zip`].
pub struct Zipped<A, B, TA, TB> {
    a: A,
    b: B,
    _marker: std::marker::PhantomData<fn() -> (TA, TB)>,
}

impl<A, B, TA, TB> Clone for Zipped<A, B, TA, TB>
where
    A: Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<A, B, TA, TB> fmt::Debug for Zipped<A, B, TA, TB>
where
    A: Signal<TA>,
    B: Signal<TB>,
    TA: Clone + fmt::Debug + 'static,
    TB: Clone + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zipped")
            .field("value", &self.get())
            .finish()
    }
}

impl<A, B, TA, TB> Signal<(TA, TB)> for Zipped<A, B, TA, TB>
where
    A: Signal<TA>,
    B: Signal<TB>,
    TA: Clone + 'static,
    TB: Clone + 'static,
{
    fn get(&self) -> (TA, TB) {
        (self.a.get(), self.b.get())
    }

    fn with<V>(&self, f: impl FnOnce(&(TA, TB)) -> V) -> V {
        let value = self.get();
        f(&value)
    }

    fn set(&self, _value: (TA, TB)) {
        read_only_panic("quoin_core::zip");
    }

    fn update(&self, _f: impl FnOnce(&mut (TA, TB))) {
        read_only_panic("quoin_core::zip");
    }
}

// ---------------------------------------------------------------------------
// Memoized derived signals
// ---------------------------------------------------------------------------

/// A [`Derived`] signal that caches its last output until the source value
/// changes (as determined by `PartialEq`).
///
/// Plain [`derived`] recomputes the closure on *every* read — correct and
/// side-effect free, but wasteful when the mapping is expensive and reads are
/// frequent. `Memo` keeps the last `(input, output)` pair behind a mutex and
/// only re-invokes `f` after a source read observes a different value.
///
/// # Trade-offs
///
/// - Each read clones the source value to compare it: prefer `Memo` when
///   `T: PartialEq` is cheaper than `U = F(T)` (formatting, sorting, filtering).
/// - The cache is process-local to this handle (clones share it through the
///   inner `Arc`), so it is safe to pass around.
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// let source = quoin_core::test_support::static_signal(2u32);
/// let calls = Arc::new(AtomicUsize::new(0));
/// let calls2 = calls.clone();
/// let slow = quoin_core::memo(&source, move |v: &u32| {
///     calls2.fetch_add(1, Ordering::Relaxed);
///     v * v
/// });
/// assert_eq!(slow.get(), 4);
/// assert_eq!(slow.get(), 4); // cached: closure not re-run
/// assert_eq!(calls.load(Ordering::Relaxed), 1);
/// source.set(3);
/// assert_eq!(slow.get(), 9); // source changed: recompute
/// assert_eq!(calls.load(Ordering::Relaxed), 2);
/// ```
pub struct Memo<S, T, U, F> {
    source: S,
    f: F,
    cache: Arc<Mutex<Option<(T, U)>>>,
}

impl<S, T, U, F> Clone for Memo<S, T, U, F>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            f: self.f.clone(),
            cache: Arc::clone(&self.cache),
        }
    }
}

impl<S, T, U, F> fmt::Debug for Memo<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + PartialEq + 'static,
    U: Clone + fmt::Debug + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo").field("value", &self.get()).finish()
    }
}

/// Creates a memoized derived signal: like [`derived`], but the mapping
/// closure only re-runs when the source value has changed since the last
/// read (compared via `PartialEq`).
///
/// # Panics
///
/// Panics if the internal cache mutex is poisoned (a prior panic while
/// computing a value).
pub fn memo<S, T, U, F>(source: &S, f: F) -> Memo<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + PartialEq + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    Memo {
        source: source.clone(),
        f,
        cache: Arc::new(Mutex::new(None)),
    }
}

impl<S, T, U, F> Memo<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + PartialEq + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    /// Returns a reference to the underlying source signal.
    #[must_use]
    pub const fn source(&self) -> &S {
        &self.source
    }

    /// Drops the cached value so the next read recomputes.
    ///
    /// Only needed when the mapping closure is impure (e.g. depends on wall
    /// time); value-keyed invalidation is automatic otherwise.
    ///
    /// # Panics
    ///
    /// Panics if the internal cache mutex is poisoned (a prior panic while
    /// computing a value).
    pub fn invalidate(&self) {
        *self.cache.lock().unwrap() = None;
    }

    fn compute(&self) -> U {
        let input = self.source.get();
        let mut cache = self.cache.lock().unwrap();
        if let Some((last_in, last_out)) = cache.as_ref()
            && *last_in == input
        {
            return last_out.clone();
        }
        let output = (self.f)(&input);
        *cache = Some((input, output.clone()));
        output
    }
}

impl<S, T, U, F> Signal<U> for Memo<S, T, U, F>
where
    S: Signal<T>,
    T: Clone + PartialEq + 'static,
    U: Clone + 'static,
    F: Fn(&T) -> U + Clone + 'static,
{
    fn get(&self) -> U {
        self.compute()
    }

    fn with<V>(&self, f: impl FnOnce(&U) -> V) -> V {
        let value = self.compute();
        f(&value)
    }

    fn set(&self, _value: U) {
        read_only_panic("quoin_core::Memo");
    }

    fn update(&self, _f: impl FnOnce(&mut U)) {
        read_only_panic("quoin_core::Memo");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SignalExt;
    use crate::test_support::static_signal;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn derived_computes_from_source() {
        let source = static_signal(10u32);
        let d = derived(&source, |v| v * 3);
        assert_eq!(d.get(), 30);
        source.set(5);
        assert_eq!(d.get(), 15);
    }

    #[test]
    fn derived_recomputes_on_every_read() {
        let source = static_signal(1u32);
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let d = derived(&source, move |v: &u32| {
            calls2.fetch_add(1, Ordering::SeqCst);
            v + 1
        });
        assert_eq!(d.get(), 2);
        assert_eq!(d.get(), 2);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "no caching: recompute per read"
        );
    }

    #[test]
    fn derived_chains() {
        let source = static_signal(2i32);
        let d = derived(&source, |v| v * 10).map(|v| v + 1);
        assert_eq!(d.get(), 21);
        source.set(0);
        assert_eq!(d.get(), 1);
    }

    #[test]
    fn derived_with_avoids_clone_of_source_value() {
        let source = static_signal(vec![1u8, 2, 3]);
        let d = derived(&source, |v: &Vec<u8>| v.len());
        let out = d.with(|len| *len);
        assert_eq!(out, 3);
    }

    #[test]
    fn zip_combines_two_signals() {
        let a = static_signal(1u32);
        let b = static_signal("x".to_string());
        let z = zip(&a, &b);
        let (num, s) = z.get();
        assert_eq!(num, 1);
        assert_eq!(s, "x");
        a.set(9);
        let (num, s) = z.get();
        assert_eq!(num, 9);
        assert_eq!(s, "x");
    }

    #[test]
    fn derived_is_clone() {
        let source = static_signal(7u32);
        let d = derived(&source, std::string::ToString::to_string);
        let d2 = d.clone();
        assert_eq!(d2.get(), "7");
        source.set(8);
        assert_eq!(d.get(), "8");
        assert_eq!(d2.get(), "8");
    }

    #[test]
    fn derived_source_accessor() {
        let source = static_signal(3u32);
        let d: Derived<_, u32, u32, _> = derived(&source, |v: &u32| *v);
        assert_eq!(d.source().get(), 3);
    }

    #[test]
    fn derived_debug_shows_computed_value() {
        let source = static_signal(4u32);
        let d = derived(&source, |v| v * 2);
        assert_eq!(format!("{d:?}"), "Derived { value: 8 }");
    }

    #[test]
    fn derived_set_and_update_panic_with_clear_message() {
        let source = static_signal(1u32);
        let d = derived(&source, |v| *v);
        let message = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| d.set(2)))
            .err()
            .and_then(|payload| {
                payload.downcast_ref::<String>().cloned().or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(std::string::ToString::to_string)
                })
            })
            .expect("set on Derived must panic");
        assert!(message.contains("read-only"), "got: {message}");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            d.update(|_| {});
        }));
        assert!(result.is_err(), "update on Derived must panic");
    }

    #[test]
    fn zip_set_and_update_panic() {
        let a = static_signal(1u32);
        let b = static_signal(2u32);
        let z = zip(&a, &b);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| z.set((3, 4))));
        assert!(result.is_err(), "set on Zipped must panic");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| z.update(|_| {})));
        assert!(result.is_err(), "update on Zipped must panic");
    }

    #[test]
    fn memo_caches_until_source_changes() {
        let source = static_signal(2u32);
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let slow = memo(&source, move |v: &u32| {
            calls2.fetch_add(1, Ordering::Relaxed);
            v * v
        });
        assert_eq!(slow.get(), 4);
        assert_eq!(slow.get(), 4);
        assert_eq!(slow.get(), 4);
        assert_eq!(
            calls.load(Ordering::Relaxed),
            1,
            "cache must suppress reruns"
        );
        source.set(3);
        assert_eq!(slow.get(), 9);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        // Same value written again: cached result still valid.
        source.set(3);
        assert_eq!(slow.get(), 9);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn memo_invalidate_forces_recompute() {
        let source = static_signal(5u32);
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let m = memo(&source, move |v: &u32| {
            calls2.fetch_add(1, Ordering::Relaxed);
            *v
        });
        assert_eq!(m.get(), 5);
        m.invalidate();
        assert_eq!(m.get(), 5);
        assert_eq!(
            calls.load(Ordering::Relaxed),
            2,
            "invalidate drops the cache"
        );
    }

    #[test]
    fn memo_clones_share_cache() {
        let source = static_signal(1u32);
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let m = memo(&source, move |v: &u32| {
            calls2.fetch_add(1, Ordering::Relaxed);
            *v + 100
        });
        let m2 = m.clone();
        assert_eq!(m.get(), 101);
        assert_eq!(m2.get(), 101);
        assert_eq!(calls.load(Ordering::Relaxed), 1, "clone shares the cache");
    }

    #[test]
    fn memo_source_accessor() {
        let source = static_signal(9u32);
        let m = memo(&source, |v: &u32| *v);
        assert_eq!(m.source().get(), 9);
    }

    #[test]
    fn memo_chains_with_map() {
        let source = static_signal(3u32);
        let chained = memo(&source, |v: &u32| v * 2).map(|v| v + 1);
        assert_eq!(chained.get(), 7);
        source.set(4);
        assert_eq!(chained.get(), 9);
    }

    #[test]
    fn memo_debug_shows_computed_value() {
        let source = static_signal(4u32);
        let m = memo(&source, |v| v * 2);
        assert_eq!(format!("{m:?}"), "Memo { value: 8 }");
    }
}
