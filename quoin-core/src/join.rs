//! Shared join-handle machinery for the framework adapters.
//!
//! Every adapter's `Executor` returns a join handle with the same lifecycle
//! contract (see [`JoinHandle`]): await for the result, `abort` to cancel,
//! `detach` for fire-and-forget, and abort-on-drop for structured
//! concurrency. That contract used to be hand-written five times with only
//! the "how do we cancel the underlying task" part differing per framework.
//!
//! This module factors the shared 80% into [`SharedJoinHandle`], generic over
//! a [`RawTask`] that models the framework-specific cancellation:
//!
//! - [`AbortTask`] — a `futures::future::AbortHandle` around a pooled future
//!   (Leptos, Dioxus, Floem). Abort is explicit; dropping the handle does not
//!   cancel anything by itself.
//! - GPUI and Xilem define their own [`RawTask`] impls over their native task
//!   types (`gpui::Task` / `tokio::task::JoinHandle`), whose ownership rules
//!   differ (GPUI tasks die with the handle unless detached; tokio tasks do
//!   not).
//!
//! The adapters expose their historical type names as aliases over
//! [`SharedJoinHandle`], so the public API is unchanged.

use crate::executor::JoinHandle;
use futures_channel::oneshot;
use futures_util::future::{AbortHandle, Abortable};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// The framework-specific half of a [`SharedJoinHandle`]: how to cancel and
/// how to release the underlying task.
///
/// Both methods take `&self` so the shared handle can abort from a shared
/// reference (the `JoinHandle::abort` signature) and from `Drop`. Task state
/// that must be taken out (GPUI's `Task`, tokio's `JoinHandle`) is stored
/// inside the impl behind a mutex.
pub trait RawTask: Send + Sync + 'static {
    /// Cancels the underlying task at its next suspension point (or
    /// immediately, depending on the framework). Idempotent: must be safe to
    /// call after the task already finished.
    fn abort(&self);

    /// Releases ownership so the task keeps running even when the shared
    /// handle is gone. For runtimes where tasks are independent of their
    /// handles (tokio, the pooled `Abortable` wrapper) this is a no-op.
    fn detach(&self);
}

/// [`RawTask`] for a pooled future wrapped in `futures::future::Abortable`
/// (the Leptos/Dioxus/Floem execution model, native and wasm).
#[derive(Debug)]
pub struct AbortTask(AbortHandle);

impl RawTask for AbortTask {
    fn abort(&self) {
        self.0.abort();
    }

    fn detach(&self) {}
}

/// Shared implementation of the [`JoinHandle`] contract.
///
/// Structure: a oneshot receiver carries the task result, a [`RawTask`]
/// cancels/releases the framework task, and a `detached` flag implements the
/// abort-on-drop rule. All three lifecycle methods (await, abort, detach)
/// plus `Drop` are implemented exactly once here.
///
/// The abort operation is captured once at construction as a boxed closure:
/// `Drop` implementations cannot carry trait bounds, so this is what lets an
/// unbounded `Drop` still cancel the framework task.
pub struct SharedJoinHandle<T, H> {
    inner: Arc<HandleInner<T, H>>,
}

struct HandleInner<T, H> {
    task: Arc<H>,
    abort_fn: Box<dyn Fn() + Send + Sync>,
    rx: Mutex<Option<oneshot::Receiver<T>>>,
    detached: AtomicBool,
}

impl<T, H> SharedJoinHandle<T, H>
where
    H: RawTask,
{
    /// Builds a handle from the framework task and the receiver that will
    /// carry its result. Intended for adapter crates implementing
    /// [`Executor::spawn`](crate::Executor::spawn).
    pub fn new(task: H, rx: oneshot::Receiver<T>) -> Self {
        let task = Arc::new(task);
        let for_abort = Arc::clone(&task);
        Self {
            inner: Arc::new(HandleInner {
                task,
                abort_fn: Box::new(move || for_abort.abort()),
                rx: Mutex::new(Some(rx)),
                detached: AtomicBool::new(false),
            }),
        }
    }
}

impl<T, H> SharedJoinHandle<T, H> {
    /// Whether the handle was detached (or awaited) and will therefore not
    /// cancel the task when dropped.
    #[must_use]
    pub fn is_detached(&self) -> bool {
        self.inner.detached.load(Ordering::SeqCst)
    }
}

impl<T, H> JoinHandle<T> for SharedJoinHandle<T, H>
where
    T: Send + 'static,
    H: RawTask,
{
    fn abort(&self) {
        (self.inner.abort_fn)();
    }

    fn detach(self) {
        self.inner.task.detach();
        self.inner.detached.store(true, Ordering::SeqCst);
    }
}

impl<T, H> std::future::IntoFuture for SharedJoinHandle<T, H>
where
    T: Send + 'static,
    H: RawTask,
{
    type Output = Result<T, oneshot::Canceled>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Awaiting resolves the handle: the task must not be cancelled by
            // the drop that follows, even if the task already finished.
            self.inner.detached.store(true, Ordering::SeqCst);
            let rx = self.inner.rx.lock().unwrap().take();
            match rx {
                Some(rx) => rx.await,
                // A second await after the receiver was taken can only
                // resolve with `Canceled`; the task itself keeps running.
                None => Err(oneshot::Canceled),
            }
        })
    }
}

impl<T, H> Drop for SharedJoinHandle<T, H> {
    fn drop(&mut self) {
        // Structured concurrency: dropping the handle without awaiting or
        // detaching cancels the underlying task.
        if !self.inner.detached.load(Ordering::SeqCst) {
            (self.inner.abort_fn)();
        }
    }
}

impl<T, H> std::fmt::Debug for SharedJoinHandle<T, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedJoinHandle")
            .field("detached", &self.is_detached())
            .finish_non_exhaustive()
    }
}

/// The join handle used by adapters that wrap their future in an
/// `Abortable` on the shared worker pool (Leptos, Dioxus, Floem).
pub type AbortJoinHandle<T> = SharedJoinHandle<T, AbortTask>;

/// Spawns `future` wrapped in an `Abortable` and returns the shared
/// [`AbortJoinHandle`].
///
/// On native targets the future runs on quoin's shared worker pool; on
/// `wasm32` it runs on the browser's main thread. This is the complete
/// `Executor::spawn` body for the Leptos, Dioxus, and Floem adapters —
/// previously duplicated in all three.
pub fn spawn_abortable<F>(future: F) -> AbortJoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(result) = Abortable::new(future, abort_registration).await {
                let _ = tx.send(result);
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::pool::spawn_pooled(async move {
            if let Ok(result) = Abortable::new(future, abort_registration).await {
                let _ = tx.send(result);
            }
        });
    }

    SharedJoinHandle::new(AbortTask(abort_handle), rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::block_on;
    use std::future::IntoFuture as _;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    /// Spawns a job on the pool that waits for a worker-thread signal
    /// (after `delay_ms`) and then bumps `counter` and delivers `()` to the
    /// handle. The shared abortable handle can cancel it any time before the
    /// signal fires.
    fn spawn_signalled_job(counter: Arc<AtomicUsize>, delay_ms: u64) -> AbortJoinHandle<()> {
        let (sig_tx, sig_rx) = oneshot::channel::<()>();
        let (abort, registration) = AbortHandle::new_pair();
        let (tx, rx) = oneshot::channel::<()>();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(delay_ms));
            let _ = sig_tx.send(());
        });
        crate::pool::spawn_pooled(async move {
            let completed = Abortable::new(
                async {
                    let _ = sig_rx.await;
                },
                registration,
            )
            .await;
            if completed.is_ok() {
                counter.fetch_add(1, Ordering::SeqCst);
                let _ = tx.send(());
            }
        });
        SharedJoinHandle::new(AbortTask(abort), rx)
    }

    #[test]
    fn await_resolves_with_result_and_marks_detached() {
        let handle = spawn_abortable(async { 40 + 2 });
        assert!(!handle.is_detached());
        assert_eq!(block_on(handle.into_future()).expect("task result"), 42);
    }

    #[test]
    fn abort_resolves_await_with_canceled() {
        // `pending` never completes on its own, so the abort must win.
        let handle = spawn_abortable(std::future::pending::<u32>());
        handle.abort();
        assert!(
            block_on(handle.into_future()).is_err(),
            "aborted task resolves Canceled"
        );
    }

    #[test]
    fn drop_without_detach_cancels_the_task() {
        let counter = Arc::new(AtomicUsize::new(0));
        let handle = spawn_signalled_job(Arc::clone(&counter), 50);
        // Cancel before the signal thread fires.
        drop(handle);
        std::thread::sleep(Duration::from_millis(120));
        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "dropped handle must cancel its task (structured concurrency)"
        );
    }

    #[test]
    fn detach_lets_the_task_finish() {
        let counter = Arc::new(AtomicUsize::new(0));
        let handle = spawn_signalled_job(Arc::clone(&counter), 30);
        handle.detach();
        std::thread::sleep(Duration::from_millis(120));
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "detached task must run to completion"
        );
    }

    #[test]
    fn abort_after_completion_is_harmless() {
        let counter = Arc::new(AtomicUsize::new(0));
        let handle = spawn_signalled_job(Arc::clone(&counter), 5);
        // Wait until the pooled job actually finished.
        let deadline = Instant::now() + Duration::from_secs(2);
        while counter.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        handle.abort();
        let result = block_on(handle.into_future());
        assert!(
            result.is_ok(),
            "aborting an already-finished task keeps the result available"
        );
    }

    #[test]
    fn abort_is_idempotent() {
        let handle = spawn_abortable(std::future::pending::<u32>());
        handle.abort();
        handle.abort();
        // A shared reference is enough: `abort(&self)` must stay callable
        // repeatedly (e.g. from several UI callbacks).
        let same: &AbortJoinHandle<u32> = &handle;
        same.abort();
        assert!(block_on(handle.into_future()).is_err());
    }

    #[test]
    fn abort_task_raw_impl_aborts_pooled_future() {
        let (tx, rx) = oneshot::channel::<u32>();
        let (abort, registration) = AbortHandle::new_pair();
        crate::pool::spawn_pooled(async move {
            if Abortable::new(std::future::pending::<()>(), registration).await == Ok(()) {
                let _ = tx.send(1);
            }
        });
        let handle = SharedJoinHandle::new(AbortTask(abort), rx);
        handle.abort();
        assert!(block_on(handle.into_future()).is_err());
    }

    #[test]
    fn abort_through_trait_reference() {
        let handle = spawn_abortable(async { 7u32 });
        // `abort(&self)` through the trait object path to prove the shared
        // implementation satisfies the `JoinHandle` contract for references.
        JoinHandle::<u32>::abort(&handle);
        assert!(block_on(handle.into_future()).is_err());
    }
}
