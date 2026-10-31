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
        panic!(
            "quoin_core::Derived is read-only; mutate the source signal instead. \
             If you need a writable projection, create two signals and keep them \
             in sync from the writer side."
        );
    }

    fn update(&self, _f: impl FnOnce(&mut U)) {
        panic!(
            "quoin_core::Derived is read-only; mutate the source signal instead. \
             If you need a writable projection, create two signals and keep them \
             in sync from the writer side."
        );
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
    pub fn source(&self) -> &S {
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
    pub fn map<V, G>(self, g: G) -> Derived<Derived<S, T, U, F>, U, V, G>
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
        panic!("quoin_core::zip produces a read-only signal");
    }

    fn update(&self, _f: impl FnOnce(&mut (TA, TB))) {
        panic!("quoin_core::zip produces a read-only signal");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let d = derived(&source, move |v| {
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
}
