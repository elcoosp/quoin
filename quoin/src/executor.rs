use std::future::Future;

/// A handle to a spawned asynchronous task.
pub trait JoinHandle<T>: Send + Sync {
    /// Aborts the task.
    fn abort(&self);
}

/// A framework's async task executor.
pub trait Executor: Clone + Send + Sync + 'static {
    /// The framework's native join handle type for a spawned task.
    type JoinHandle<T: Send + 'static>: JoinHandle<T>;

    /// Spawns a future on the executor.
    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
