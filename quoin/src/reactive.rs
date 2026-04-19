use crate::{Executor, Signal};

/// A framework-specific reactive runtime context.
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    /// The framework's native signal type.
    type Signal<T: Clone + 'static>: Signal<T>;

    /// The framework's async executor.
    type Executor: Executor;

    /// Creates a new signal with the given initial value.
    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T>;

    /// Returns the executor for spawning asynchronous tasks.
    fn executor(&self) -> Self::Executor;

    /// Requests that the UI re-render.
    fn request_update(&self);
}
