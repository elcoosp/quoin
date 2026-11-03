//! `!Send` task execution for single-threaded runtimes (WASM, main-thread UI).
//!
//! Browsers are single-threaded from the WASM perspective, and most DOM-touching
//! futures are intentionally `!Send`. The [`Executor`] trait requires `Send`
//! futures so native adapters can move work between threads; this module adds
//! the counterpart for local tasks:
//!
//! - **On `wasm32`** — tasks run on the browser main thread via
//!   `wasm_bindgen_futures::spawn_local`, exactly like the platform expects.
//! - **On native** — a `!Send` future cannot soundly cross threads, so
//!   [`spawn_local_task`] pushes the task onto a **thread-local FIFO queue**;
//!   call [`run_local_tasks`] to drive queued tasks to completion (to their
//!   next completion, one at a time). This is deterministic, which makes it
//!   excellent for tests and for `quoin-testkit`-based suites.
//!
//! # Lifecycle semantics
//!
//! Local tasks mirror [`Executor::spawn`]: `abort` stops the task at its next
//! suspension point, `detach` opts out of drop-cancellation, and dropping the
//! [`LocalTask`] handle cancels the task (structured concurrency).
//!
//! # Notes
//!
//! - Native tasks run **one at a time** inside [`run_local_tasks`]; a task
//!   awaiting another queued task's completion must be drained in a later
//!   call (or will deadlock if the dependency is earlier in the queue).
//! - GPUI does not implement `LocalExecutor` — its executor is `!Send` by
//!   design; use `Executor::spawn`.
//!
//! [`Executor`]: crate::Executor
//! [`Executor::spawn`]: crate::Executor::spawn

use futures_util::future::{AbortHandle, Abortable};
#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};

/// A handle to a spawned `!Send` (local) task.
///
/// Mirrors [`JoinHandle`](crate::JoinHandle) without the `Send + Sync`
/// supertraits — a local task's channel may carry `!Send` values, so the
/// handle itself cannot cross threads.
pub trait LocalJoinHandle<T>: 'static {
    /// Aborts the task at its next suspension point.
    fn abort(&self);

    /// Detaches the task so it runs to completion even if this handle is
    /// dropped. The task's result is discarded.
    fn detach(self)
    where
        Self: Sized;
}

/// A framework executor that can also run `!Send` futures.
///
/// Implementations must run the future **on the current thread** (queued on
/// native, platform-driven on `wasm32`).
pub trait LocalExecutor: Clone + Send + Sync + 'static {
    /// Handle type for a spawned local task.
    type JoinHandle<T: 'static>: LocalJoinHandle<T>;

    /// Spawns a `!Send` future.
    ///
    /// Unlike [`Executor::spawn`](crate::Executor::spawn), neither the future
    /// nor its output needs to be `Send`. The future still must be `'static`.
    fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static;
}

// ---------------------------------------------------------------------------
// Shared reference implementation
// ---------------------------------------------------------------------------

/// Ready-made local task handle used by the adapters' `LocalExecutor`
/// implementations, so every framework shares identical abort/detach/drop
/// semantics.
pub struct LocalTask<T> {
    abort: AbortHandle,
    rx: Option<futures_channel::oneshot::Receiver<T>>,
    detached: AtomicBool,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: 'static> LocalJoinHandle<T> for LocalTask<T> {
    fn abort(&self) {
        self.abort.abort();
    }

    fn detach(self) {
        self.detached.store(true, Ordering::SeqCst);
    }
}

impl<T> Drop for LocalTask<T> {
    fn drop(&mut self) {
        // Structured concurrency: dropping without awaiting/detaching
        // cancels the task.
        if !self.detached.load(Ordering::SeqCst) {
            self.abort.abort();
        }
    }
}

impl<T: 'static> Future for LocalTask<T> {
    type Output = Result<T, futures_channel::oneshot::Canceled>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = Pin::into_inner(self);
        // Taking the receiver marks the handle "awaited": the drop guard
        // must not cancel a task whose result is being consumed.
        if let Some(mut rx) = this.rx.take() {
            this.detached.store(true, Ordering::SeqCst);
            return Pin::new(&mut rx).poll(cx);
        }
        std::task::Poll::Ready(Err(futures_channel::oneshot::Canceled))
    }
}

// ---------------------------------------------------------------------------
// Thread-local queue (native)
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
struct QueuedJob {
    /// Runs the wrapped task to completion when invoked.
    run: Box<dyn FnOnce() + 'static>,
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static LOCAL_QUEUE: RefCell<VecDeque<QueuedJob>> = const { RefCell::new(VecDeque::new()) };
}

/// Drives all queued local tasks on the **current thread** to completion,
/// one at a time, and returns how many ran.
///
/// Returns `0` when the queue is empty (or on `wasm32`, where the platform
/// event loop drives tasks instead).
///
/// On `wasm32` this is a no-op (the platform event loop drives local tasks
/// instead).
///
/// # Example
///
/// ```
/// use quoin_core::prelude::*;
///
/// let task = quoin_core::spawn_local_task(async { 6 * 7 });
/// // Not started yet on native — drive the queue:
/// assert_eq!(quoin_core::local::run_local_tasks(), 1);
/// let value = quoin_core::pool::block_on(task);
/// assert_eq!(value, Ok(42));
/// ```
#[must_use]
pub fn run_local_tasks() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        0 // platform-driven; nothing queued locally
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut ran = 0;
        loop {
            let job = LOCAL_QUEUE.with(|q| q.borrow_mut().pop_front());
            match job {
                Some(job) => {
                    (job.run)();
                    ran += 1;
                }
                None => break,
            }
        }
        ran
    }
}

/// Returns the number of local tasks queued on the current thread.
#[must_use]
pub fn pending_local_tasks() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        LOCAL_QUEUE.with(|q| q.borrow().len())
    }
}

/// Spawns a `!Send` future: on `wasm32` it runs on the platform event loop;
/// on native it is queued on the current thread until [`run_local_tasks`]
/// drains it.
///
/// This is the shared engine behind the adapters' `LocalExecutor`
/// implementations — one audited implementation instead of four.
pub fn spawn_local_task<F>(future: F) -> LocalTask<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    let (tx, rx) = futures_channel::oneshot::channel();
    let (abort, registration) = AbortHandle::new_pair();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(result) = Abortable::new(future, registration).await {
                let _ = tx.send(result);
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let job = async move {
            if let Ok(result) = Abortable::new(future, registration).await {
                let _ = tx.send(result);
            }
        };
        LOCAL_QUEUE.with(|q| {
            q.borrow_mut().push_back(QueuedJob {
                run: Box::new(move || crate::pool::block_on(job)),
            });
        });
    }

    LocalTask {
        abort,
        rx: Some(rx),
        detached: AtomicBool::new(false),
        _marker: std::marker::PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::sync::atomic::AtomicUsize;

    /// A `!Send` future: holds an `Rc`.
    struct HoldsRc(Rc<AtomicUsize>);
    impl Future for HoldsRc {
        type Output = u32;
        fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<u32> {
            std::task::Poll::Ready(
                u32::try_from(self.0.load(std::sync::atomic::Ordering::SeqCst)).unwrap_or(u32::MAX),
            )
        }
    }

    #[test]
    fn local_task_runs_non_send_future() {
        let task = spawn_local_task(async {
            let rc = Rc::new(AtomicUsize::new(41));
            let rc2 = rc.clone();
            async move {
                rc2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            .await;
            HoldsRc(rc).await
        });
        assert_eq!(run_local_tasks(), 1);
        let result = crate::pool::block_on(task);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn local_task_abort_cancels() {
        let task = spawn_local_task(async {
            // Long-running !Send future
            let rc = Rc::new(());
            std::future::pending::<()>().await;
            let _ = rc;
            1u32
        });
        task.abort();
        assert_eq!(
            run_local_tasks(),
            1,
            "aborted job still runs, stops immediately"
        );
        let result = crate::pool::block_on(task);
        assert!(
            result.is_err(),
            "aborted local task must resolve with Canceled"
        );
    }

    #[test]
    fn local_task_drop_cancels_and_detach_lets_run() {
        // drop => cancel
        {
            let task = spawn_local_task(async { 1u32 });
            drop(task);
        }
        assert_eq!(
            run_local_tasks(),
            1,
            "dropped task is cancelled but still runs to abort"
        );
        // detach => completes even though handle gone
        let flag = std::sync::Arc::new(AtomicUsize::new(0));
        let flag2 = flag.clone();
        let task = spawn_local_task(async move {
            flag2.store(1, std::sync::atomic::Ordering::SeqCst);
            1u32
        });
        task.detach();
        assert_eq!(run_local_tasks(), 1);
        assert_eq!(flag.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn tasks_run_in_fifo_order() {
        let order = Rc::new(RefCell::new(Vec::new()));
        let o1 = order.clone();
        let _t1 = spawn_local_task(async move {
            o1.borrow_mut().push(1);
        });
        let o2 = order.clone();
        let _t2 = spawn_local_task(async move {
            o2.borrow_mut().push(2);
        });
        assert_eq!(pending_local_tasks(), 2);
        assert_eq!(run_local_tasks(), 2);
        assert_eq!(*order.borrow(), vec![1, 2]);
        assert_eq!(pending_local_tasks(), 0);
    }

    #[test]
    fn empty_queue_reports_zero() {
        let _drained = run_local_tasks(); // drain leftovers from other tests on this thread
        assert_eq!(pending_local_tasks(), 0);
        assert_eq!(run_local_tasks(), 0);
    }
}
