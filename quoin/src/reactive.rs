use crate::{Executor, Signal};

pub trait ReactiveContext: Clone + Send + Sync + 'static {
    type Signal<T: 'static>: Signal<T>;
    type Executor: Executor;
    fn create_signal<T: 'static>(&self, initial: T) -> Self::Signal<T>;
    fn executor(&self) -> Self::Executor;
    fn request_update(&self);
}
