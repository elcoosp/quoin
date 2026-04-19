use std::future::Future;

pub trait JoinHandle<T>: Send + Sync {
    fn abort(&self);
}

pub trait Executor: Clone + Send + Sync + 'static {
    type JoinHandle<T: Send + 'static>: JoinHandle<T>;
    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
