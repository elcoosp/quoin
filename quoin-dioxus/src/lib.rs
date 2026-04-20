use dioxus::prelude::*;
use quoin::{Executor, JoinHandle, ReactiveContext, Signal};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

#[derive(Clone)]
pub struct DioxusContext;

impl DioxusContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for DioxusContext {
    type Signal<T: Clone + 'static> = DioxusSignal<T>;
    type Executor = DioxusExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        let signal = dioxus::prelude::Signal::new(initial);
        DioxusSignal {
            signal: Rc::new(RefCell::new(signal)),
        }
    }

    fn executor(&self) -> Self::Executor {
        DioxusExecutor
    }

    fn request_update(&self) {
        // Dioxus reactivity is automatic.
    }
}

#[derive(Clone)]
pub struct DioxusSignal<T: Clone + 'static> {
    signal: Rc<RefCell<dioxus::prelude::Signal<T>>>,
}

impl<T: Clone + 'static> Signal<T> for DioxusSignal<T> {
    fn get(&self) -> T {
        self.signal.borrow().read().clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        f(&self.signal.borrow().read())
    }

    fn set(&self, value: T) {
        // RefCell allows us to take a mutable reference to the signal
        // even though `self` is immutable, satisfying Dioxus's `&mut self` requirement.
        self.signal.borrow_mut().set(value);
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        // Borrow the signal mutably, then call Dioxus's `write()` to get a mutable guard
        let mut borrow = self.signal.borrow_mut();
        let mut write = borrow.write();
        f(&mut *write);
    }
}

#[derive(Clone)]
pub struct DioxusExecutor;

impl Executor for DioxusExecutor {
    type JoinHandle<T: Send + 'static> = DioxusJoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = futures::channel::oneshot::channel();

        std::thread::spawn(move || {
            let result = futures::executor::block_on(future);
            let _ = tx.send(result);
        });

        DioxusJoinHandle { rx: Some(rx) }
    }
}

pub struct DioxusJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for DioxusJoinHandle<T> {
    fn abort(&self) {}
}

impl<T: Send + 'static> std::future::IntoFuture for DioxusJoinHandle<T> {
    type Output = Result<T, futures::channel::oneshot::Canceled>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            if let Some(rx) = self.rx.take() {
                rx.await
            } else {
                unreachable!("Receiver should be set")
            }
        })
    }
}
