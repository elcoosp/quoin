use floem_reactive::{RwSignal, SignalGet, SignalUpdate, SignalWith};
use quoin_core::{Executor, JoinHandle, ReactiveContext, Signal};
use send_wrapper::SendWrapper;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

thread_local! {
    static GLOBAL_STORE: std::cell::RefCell<HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[derive(Clone, Default)]
pub struct FloemContext;

impl FloemContext {
    pub fn new() -> Self {
        Self
    }
}

impl ReactiveContext for FloemContext {
    type Signal<T: Clone + 'static> = FloemSignal<T>;
    type Executor = FloemExecutor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        FloemSignal::new(initial)
    }

    fn executor(&self) -> Self::Executor {
        FloemExecutor
    }

    fn request_update(&self) {
        // Floem's reactivity is automatic.
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        GLOBAL_STORE.with(|store| {
            store.borrow_mut().insert(TypeId::of::<T>(), Box::new(value));
        });
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        GLOBAL_STORE.with(|store| {
            store
                .borrow()
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
                .cloned()
                .map(|v| FloemSignal::new(v))
        })
    }
}

#[derive(Clone)]
pub struct FloemSignal<T: Clone + 'static> {
    inner: RwSignal<SendWrapper<T>>,
}

impl<T: Clone + 'static> FloemSignal<T> {
    fn new(initial: T) -> Self {
        Self {
            inner: RwSignal::new(SendWrapper::new(initial)),
        }
    }
}

impl<T: Clone + 'static> Signal<T> for FloemSignal<T> {
    fn get(&self) -> T {
        (*self.inner.get()).clone()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.inner.with(|wrapper| f(&**wrapper))
    }

    fn set(&self, value: T) {
        self.inner.set(SendWrapper::new(value));
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        self.inner.update(|wrapper| f(&mut **wrapper));
    }
}

#[derive(Clone)]
pub struct FloemExecutor;

impl Executor for FloemExecutor {
    type JoinHandle<T: Send + 'static> = FloemJoinHandle<T>;

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

        FloemJoinHandle { rx: Some(rx) }
    }
}

pub struct FloemJoinHandle<T> {
    rx: Option<futures::channel::oneshot::Receiver<T>>,
}

impl<T: Send + 'static> JoinHandle<T> for FloemJoinHandle<T> {
    fn abort(&self) {}
}

impl<T: Send + 'static> std::future::IntoFuture for FloemJoinHandle<T> {
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

impl<T: Clone + std::fmt::Debug + 'static> std::fmt::Debug for FloemSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FloemSignal")
            .field("value", &self.inner.get())
            .finish()
    }
}
