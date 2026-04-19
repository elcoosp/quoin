//! Reactive value abstraction.
//!
//! This module defines the [`Signal`] trait, which represents a readable
//! reactive value. Signals are cloneable handles that can be passed across
//! threads and stored in closures.

/// A readable reactive value.
///
/// Signals are `Clone`, making them easy to pass into closures and store in
/// multiple places. They provide two ways to access the current value:
/// [`get`] (which clones the value) and [`with`] (which borrows it).
///
/// # Example
///
/// ```rust,ignore
/// use quoin::Signal;
///
/// fn print_signal<S: Signal<u32>>(signal: &S) {
///     println!("Value: {}", signal.get());
///     signal.with(|value| println!("Borrowed: {value}"));
/// }
/// ```
///
/// [`get`]: Signal::get
/// [`with`]: Signal::with
pub trait Signal<T: Clone + 'static>: Clone {
    /// Returns the current value of the signal.
    ///
    /// This method clones the inner value. If cloning is expensive, consider
    /// using [`with`] instead.
    ///
    /// [`with`]: Signal::with
    fn get(&self) -> T;

    /// Accesses the value through a closure, avoiding a clone.
    ///
    /// The closure receives a reference to the inner value. This is more
    /// efficient than [`get`] when you only need to inspect the value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// signal.with(|value| {
    ///     println!("Current count: {value}");
    /// });
    /// ```
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U;
}
