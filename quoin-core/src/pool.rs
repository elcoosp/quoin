//! Shared growable worker pool for native task execution.
//!
//! The Leptos, Dioxus, and Floem adapters historically spawned **one OS
//! thread per task** (`std::thread::spawn` inside `Executor::spawn`). That is
//! correct but wasteful: thread creation costs tens of microseconds and
//! reserves a stack per task, which dominates when an app spawns many
//! short-lived futures (typeahead searches, timers, polling loops).
//!
//! This module provides a process-wide, grow-on-demand pool:
//!
//! - Jobs are pushed onto a shared queue; idle workers pick them up.
//! - When no worker is idle and the worker count is below the cap, a new
//!   worker is spawned. Queued jobs never block: if the cap is reached the
//!   job simply waits in the queue.
//! - Workers idle for longer than `IDLE_TIMEOUT` exit on their own, so a
//!   burst of load does not leave hundreds of parked threads behind. The last
//!   worker always stays alive so the pool never has to cold-start again.
//!
//! The pool is a plain `std` implementation (no external dependencies) and is
//! shared by all adapters in the process, mirroring how a tokio runtime would
//! amortize thread costs across subsystems.

use std::collections::VecDeque;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

/// How long an idle worker waits before exiting (bounded teardown).
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Soft cap on live workers. Beyond this, jobs queue instead of spawning
/// more threads. High enough to never be the bottleneck for UI workloads.
const MAX_WORKERS: usize = 512;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Shared {
    queue: Mutex<VecDeque<Job>>,
    signal: Condvar,
    /// Number of workers currently alive.
    workers: AtomicUsize,
    /// Workers currently executing a job (used for the spawn decision).
    busy: AtomicUsize,
}

impl Shared {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            signal: Condvar::new(),
            workers: AtomicUsize::new(0),
            busy: AtomicUsize::new(0),
        }
    }
}

static POOL: OnceLock<Arc<Shared>> = OnceLock::new();

fn pool() -> &'static Arc<Shared> {
    POOL.get_or_init(|| {
        let shared = Arc::new(Shared::new());
        // Seed with two workers so the first tasks do not pay the spawn cost
        // on the UI thread.
        for _ in 0..2 {
            spawn_worker(&shared);
        }
        shared
    })
}

fn spawn_worker(shared: &Arc<Shared>) {
    shared.workers.fetch_add(1, Ordering::AcqRel);
    let shared = Arc::clone(shared);
    let builder = std::thread::Builder::new().name("quoin-pool".into());
    if builder.spawn(move || worker_loop(&shared)).is_err() {
        // Thread spawn can fail under resource exhaustion; undo the counter
        // so a later job retries spawning.
        if let Some(p) = POOL.get() {
            p.workers.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

/// Waits for the next job, holding the queue lock only for the decisions it
/// influences: the guard is local to this function, so no lock is carried
/// across job execution or worker exit.
///
/// Returns `None` when the worker should exit: the idle timeout elapsed, no
/// job appeared on the final re-check, and at least one other worker remains.
fn next_job(shared: &Arc<Shared>) -> Option<Job> {
    let mut guard = shared.queue.lock().unwrap();
    loop {
        if let Some(job) = guard.pop_front() {
            drop(guard);
            return Some(job);
        }
        let (g, _timeout) = shared.signal.wait_timeout(guard, IDLE_TIMEOUT).unwrap();
        guard = g;
        if let Some(job) = guard.pop_front() {
            drop(guard);
            return Some(job);
        }
        if shared.workers.fetch_sub(1, Ordering::AcqRel) == 1 {
            // Last worker: stay alive so the pool never becomes empty
            // and has to cold-start again.
            shared.workers.fetch_add(1, Ordering::AcqRel);
            let (g, _t) = shared.signal.wait_timeout(guard, IDLE_TIMEOUT).unwrap();
            guard = g;
            continue;
        }
        // Exit decision was made while still holding the queue lock.
        // One final re-check closes the lost-wakeup race: a producer
        // may have pushed a job between our timeout and the counter
        // decrement. If a job appeared, restore the counter and take
        // it; otherwise the lock is released on return and any later
        // producer sees the reduced worker count and grows the pool.
        if let Some(job) = guard.pop_front() {
            shared.workers.fetch_add(1, Ordering::AcqRel);
            drop(guard);
            return Some(job);
        }
        drop(guard);
        return None;
    }
}

fn worker_loop(shared: &Arc<Shared>) {
    while let Some(job) = next_job(shared) {
        shared.busy.fetch_add(1, Ordering::AcqRel);
        job();
        shared.busy.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Runs a future to completion on the shared worker pool.
///
/// The future's output is dropped; pair with a channel to deliver results
/// (this is exactly what the adapter `Executor::spawn` implementations do).
/// Unlike `std::thread::spawn`, spawning many short-lived futures amortizes a
/// small set of reusable worker threads.
///
/// # Panics
///
/// Panics only if the underlying OS refuses to spawn a thread (resource
/// exhaustion). The pool counters are restored so a later job can retry.
pub fn spawn_pooled<F>(future: F)
where
    F: Future + Send + 'static,
{
    let shared = pool().clone();

    // Decide whether to grow the pool: only when every worker is busy.
    let workers = shared.workers.load(Ordering::Acquire);
    let busy = shared.busy.load(Ordering::Acquire);
    if busy >= workers && workers < MAX_WORKERS {
        spawn_worker(&shared);
    }

    shared.queue.lock().unwrap().push_back(Box::new(move || {
        block_on(future);
    }));
    shared.signal.notify_all();
}

/// Blocks the current thread until `future` completes.
///
/// A dependency-free `block_on` built on the safe [`std::task::Wake`] trait
/// (the crate forbids `unsafe`, which rules out hand-rolled `RawWaker`s).
/// Use it on non-UI threads; blocking a framework's main/UI thread with it
/// will freeze rendering.
///
/// # Panics
///
/// Panics if the internal mutex is poisoned by another thread panicking
/// while holding it.
pub fn block_on<F: Future>(future: F) -> F::Output {
    struct FlagWaker(Arc<ParkState>);
    struct ParkState {
        woken: Mutex<bool>,
        signal: Condvar,
    }
    impl std::task::Wake for FlagWaker {
        fn wake(self: Arc<Self>) {
            let mut woken = self.0.woken.lock().unwrap();
            *woken = true;
            // Drop the mutex guard before notifying: waiters re-lock this
            // mutex on wake-up, so releasing first avoids a lock hand-off.
            drop(woken);
            self.0.signal.notify_all();
        }
    }

    let park = Arc::new(ParkState {
        woken: Mutex::new(false),
        signal: Condvar::new(),
    });
    let waker = Waker::from(Arc::new(FlagWaker(park.clone())));
    let mut cx = Context::from_waker(&waker);
    let mut future = Box::pin(future);

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                let mut woken = park.woken.lock().unwrap();
                while !*woken {
                    woken = park.signal.wait(woken).unwrap();
                }
                *woken = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn block_on_resolves_ready_future() {
        assert_eq!(block_on(async { 40 + 2 }), 42);
    }

    #[test]
    fn block_on_wakes_from_other_thread() {
        let (tx, rx) = std::sync::mpsc::channel::<u32>();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            let _ = tx.send(7);
        });
        let value = block_on(async move {
            std::future::poll_fn(|cx| {
                if let Ok(v) = rx.try_recv() {
                    return Poll::Ready(v);
                }
                cx.waker().wake_by_ref();
                Poll::Pending
            })
            .await
        });
        assert_eq!(value, 7);
    }

    #[test]
    fn pooled_futures_all_complete() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut rxs = Vec::new();
        for _ in 0..100 {
            let c = counter.clone();
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            rxs.push(rx);
            spawn_pooled(async move {
                c.fetch_add(1, Ordering::SeqCst);
                let _ = tx.send(());
            });
        }
        for rx in rxs {
            let _ = rx.recv();
        }
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn pooled_tasks_run_concurrently() {
        // Two sleeps must overlap instead of serializing, proving more than
        // one worker exists.
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let (tx2, rx2) = std::sync::mpsc::channel::<()>();
        spawn_pooled(async move {
            std::thread::sleep(Duration::from_millis(80));
            let _ = tx.send(());
        });
        spawn_pooled(async move {
            std::thread::sleep(Duration::from_millis(80));
            let _ = tx2.send(());
        });
        let start = std::time::Instant::now();
        let _ = rx.recv();
        let _ = rx2.recv();
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(150),
            "tasks must overlap on separate workers, took {elapsed:?}"
        );
    }

    #[test]
    fn pool_handles_sequential_bursts() {
        for wave in 0..5u32 {
            let (tx, rx) = std::sync::mpsc::channel::<u32>();
            spawn_pooled(async move {
                let _ = tx.send(wave);
            });
            assert_eq!(rx.recv().unwrap(), wave);
        }
    }
}
