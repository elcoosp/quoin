//! Reactive signals and traits for managing observable state.
//!
//! This module defines the core [`Signal`] trait and its implementations,
//! providing a reactive primitive that integrates with the framework's
//! change detection.

/// A readable and writable reactive value.
///
/// Signals are `Clone`, making them easy to pass into closures and store in
/// multiple places. They provide methods to read the current value and to
/// mutate it, triggering reactive updates in the framework.
///
/// # Example
///
/// ```rust,ignore
/// use quoin::Signal;
///
/// fn increment_counter<S: Signal<u32>>(signal: &S) {
///     signal.update(|value| *value += 1);
/// }
/// ```
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

    /// Sets the value of the signal.
    ///
    /// This will trigger reactive updates in the framework.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// signal.set(42);
    /// ```
    fn set(&self, value: T);

    /// Updates the value of the signal using a closure.
    ///
    /// The closure receives a mutable reference to the current value.
    /// This is more efficient than [`get`] followed by [`set`] when
    /// you need to modify the value based on its current state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// signal.update(|value| *value += 1);
    /// ```
    fn update(&self, f: impl FnOnce(&mut T));
}
