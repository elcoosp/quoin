//! Framework‑specific reactive context.
//!
//! This module defines the [`ReactiveContext`] trait, which is the primary
//! entry point for creating signals and accessing the async executor within
//! a UI framework. Adapter crates implement this trait for their specific
//! runtime.

use crate::{Executor, Signal};

/// A framework-specific reactive runtime context.
///
/// `ReactiveContext` abstracts over the reactive primitives of a UI framework.
/// It provides methods to create signals, obtain the async executor, and
/// request UI updates.
///
/// # Implementing `ReactiveContext`
///
/// Adapter crates must implement this trait for their context type. The
/// associated types `Signal` and `Executor` should be the framework's native
/// signal and executor types, respectively.
///
/// # Example
///
/// ```rust,ignore
/// use quoin::ReactiveContext;
///
/// fn create_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
///     cx.create_signal(0u32)
/// }
/// ```
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    /// The framework's native signal type.
    ///
    /// This type must implement the [`Signal`] trait and be `Clone`.
    type Signal<T: Clone + 'static>: Signal<T>;

    /// The framework's async executor.
    ///
    /// This type must implement the [`Executor`] trait.
    type Executor: Executor;

    /// Creates a new signal with the given initial value.
    ///
    /// The signal is owned by the current reactive scope. It will be
    /// automatically cleaned up when the scope is disposed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = cx.create_signal(0u32);
    /// assert_eq!(count.get(), 0);
    /// ```
    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T>;

    /// Returns the executor for spawning asynchronous tasks.
    ///
    /// Use this executor to run background work without blocking the UI.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let executor = cx.executor();
    /// executor.spawn(async {
    ///     // Long‑running operation
    /// }).detach();
    /// ```
    fn executor(&self) -> Self::Executor;

    /// Requests that the UI re‑render.
    ///
    /// Some frameworks require an explicit call to mark the UI as dirty.
    /// This method provides a hook for that purpose. In frameworks with
    /// automatic reactivity, this may be a no‑op.
    fn request_update(&self);
}
