//! Integration tests for the shared adapter primitives in `quoin-core`:
//! the join-handle lifecycle ([`quoin_core::join`]), the thread-local global
//! store ([`quoin_core::globals`]), and the shared notifier signal
//! ([`quoin_core::NotifierSignal`]).
//!
//! These exercise the modules **together** through the public API, the way
//! the framework adapters compose them, on top of the framework-free
//! `StaticContext` and the pooled `spawn_abortable` primitive.

use quoin_core::globals;
use quoin_core::join::{AbortJoinHandle, spawn_abortable};
use quoin_core::prelude::*;
use std::future::IntoFuture as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// --- Join-handle lifecycle through the public spawn path --------------------

#[test]
fn spawned_task_awaits_to_completion() {
    let handle = spawn_abortable(async { "payload".to_string() });
    assert!(!handle.is_detached(), "fresh handle is not detached");
    let result = quoin_core::pool::block_on(handle.into_future());
    assert_eq!(result.expect("task result"), "payload");
}

#[test]
fn aborted_task_yields_canceled() {
    // `pending` never completes on its own, so the abort must win the race
    // deterministically.
    let handle = spawn_abortable(std::future::pending::<u8>());
    handle.abort();
    let result = quoin_core::pool::block_on(handle.into_future());
    assert!(result.is_err(), "aborted task must resolve with Canceled");
}

#[test]
fn drop_cancels_but_detach_completes() {
    // Path 1: dropping the handle cancels the pooled task (structured
    // concurrency): the task waits for a signal that fires 30 ms later, so
    // the immediate drop aborts it before the side effect can run.
    let ran = Arc::new(AtomicUsize::new(0));
    let ran_for_task = Arc::clone(&ran);
    let (sig_tx, sig_rx) = futures::channel::oneshot::channel::<()>();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        let _ = sig_tx.send(());
    });
    let handle: AbortJoinHandle<()> = spawn_abortable(async move {
        let _ = sig_rx.await;
        ran_for_task.fetch_add(1, Ordering::SeqCst);
    });
    drop(handle);
    std::thread::sleep(Duration::from_millis(80));
    assert_eq!(ran.load(Ordering::SeqCst), 0, "dropped handle cancels task");

    // Path 2: detach lets the same shape of task finish.
    let ran2 = Arc::new(AtomicUsize::new(0));
    let counted_in_task = Arc::clone(&ran2);
    let handle2: AbortJoinHandle<()> = spawn_abortable(async move {
        std::thread::sleep(Duration::from_millis(30));
        counted_in_task.fetch_add(1, Ordering::SeqCst);
    });
    assert!(!handle2.is_detached());
    handle2.detach();
    std::thread::sleep(Duration::from_millis(80));
    assert_eq!(ran2.load(Ordering::SeqCst), 1, "detached task completes");
}

#[test]
fn await_wakes_from_another_thread() {
    let (tx, rx) = futures::channel::oneshot::channel::<u32>();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        let _ = tx.send(41);
    });
    let handle = spawn_abortable(async move { rx.await.expect("sender alive") });
    let value = quoin_core::pool::block_on(handle.into_future());
    assert_eq!(value.expect("task result"), 41);
}

// --- Globals store: context-namespaced, thread-local ------------------------

#[test]
fn globals_are_namespaced_by_context_type() {
    struct ContextOne;
    struct ContextTwo;

    #[derive(Clone)]
    struct Cell(Arc<AtomicUsize>);

    globals::insert::<ContextOne, Cell>(Cell(Arc::new(AtomicUsize::new(1))));
    globals::insert::<ContextTwo, Cell>(Cell(Arc::new(AtomicUsize::new(2))));

    let one = globals::get::<ContextOne, Cell>().expect("context one global");
    let two = globals::get::<ContextTwo, Cell>().expect("context two global");
    assert_eq!(one.0.load(Ordering::SeqCst), 1);
    assert_eq!(two.0.load(Ordering::SeqCst), 2);

    // Cloning hands out handles to the same stored value.
    one.0.store(10, Ordering::SeqCst);
    assert_eq!(two.0.load(Ordering::SeqCst), 2);
    assert_eq!(
        globals::get::<ContextOne, Cell>()
            .expect("again")
            .0
            .load(Ordering::SeqCst),
        10
    );
}

// --- NotifierSignal through the full context flow ---------------------------

#[test]
fn notifier_signal_flows_through_static_context() {
    let updates = Arc::new(AtomicUsize::new(0));
    let updates_for_signal = Arc::clone(&updates);
    let signal = quoin_core::NotifierSignal::new(0u32, move || {
        updates_for_signal.fetch_add(1, Ordering::SeqCst);
    });

    // set_eq skips equal writes (no notify) and fires on changes.
    assert!(!signal.set_eq(0));
    assert_eq!(updates.load(Ordering::SeqCst), 0);
    assert!(signal.set_eq(7));
    assert_eq!(updates.load(Ordering::SeqCst), 1);

    // A batch collapses N mutations: no per-write notifies, and the context
    // receives exactly one update request at the end.
    let cx = quoin_core::test_support::StaticContext;
    cx.batch(|| {
        signal.set(8);
        signal.update(|v| *v += 1);
    });
    assert_eq!(signal.get(), 9);
    assert_eq!(updates.load(Ordering::SeqCst), 1, "batch suppressed both");
}

#[test]
fn derived_and_memo_read_notifier_signals() {
    let source = quoin_core::NotifierSignal::new(vec![1u32, 2, 3], || {});
    let total = source.map(|values| values.iter().sum::<u32>());
    assert_eq!(total.get(), 6);
    let memoized = quoin_core::memo(&source, |values: &Vec<u32>| values.iter().sum::<u32>());
    assert_eq!(memoized.get(), 6);
    source.update(|values| values.push(4));
    assert_eq!(memoized.get(), 10);
}
