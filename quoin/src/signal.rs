/// A readable reactive value.
///
/// Signals are `Clone`, making them easy to pass into closures
/// and store in multiple places.
pub trait Signal<T: Clone + 'static>: Clone {
    /// Returns the current value of the signal.
    fn get(&self) -> T;

    /// Accesses the value through a closure, avoiding a clone.
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U;
}
