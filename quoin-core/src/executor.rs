//! Async executor abstraction for spawning tasks.
//!
//! This module defines the [`Executor`] trait, which provides a unified
//! interface for spawning futures onto the framework's native async runtime.
//! It also defines [`JoinHandle`], a handle for awaiting or aborting spawned
//! tasks.

use std::future::Future;

/// A handle to a spawned asynchronous task.
///
/// `JoinHandle` allows you to abort a running task or await its result (via
/// `IntoFuture`). It is returned by [`Executor::spawn`].
///
/// # Example
///
/// ```rust,ignore
/// let handle = executor.spawn(async { 42 });
/// let result = handle.await.unwrap();
/// assert_eq!(result, 42);
/// ```
pub trait JoinHandle<T>: Send + Sync {
    /// Aborts the task.
    ///
    /// If the task has already completed, this does nothing.
    fn abort(&self);
}

/// A framework's async task executor.
///
/// This trait provides a way to spawn `Send` futures onto the framework's
/// background thread pool. Implementations are provided by adapter crates.
///
/// # Example
///
/// ```rust,ignore
/// use quoin_core::Executor;
///
/// fn spawn_work<E: Executor>(executor: &E) {
///     executor.spawn(async {
///         // Perform background work
///     }).detach();
/// }
/// ```
pub trait Executor: Clone + Send + Sync + 'static {
    /// The framework's native join handle type for a spawned task.
    ///
    /// This type must implement [`JoinHandle`] and typically also implements
    /// `IntoFuture` so that the result can be awaited.
    type JoinHandle<T: Send + 'static>: JoinHandle<T>;

    /// Spawns a future on the executor.
    ///
    /// The future must be `Send` and its output must also be `Send`. This
    /// ensures it can be safely transferred across threads.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let handle = executor.spawn(async {
    ///     "Hello from background".to_string()
    /// });
    /// let result = handle.await.unwrap();
    /// ```
    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
