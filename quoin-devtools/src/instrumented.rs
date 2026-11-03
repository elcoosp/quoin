//! The [`Instrumented`] `ReactiveContext` wrapper and its traced signal and
//! executor types.
//!
//! Wrapping any adapter context records signal creation, mutations, update
//! requests, and task lifecycle events into an [`EventLog`], while forwarding
//! every operation unchanged to the wrapped context.

use crate::event::{EventKind, EventLog};
use crate::record::{mutation_event, record};
use quoin_core::executor::Executor;
use quoin_core::reactive::ReactiveContext;
use quoin_core::signal::Signal;
use std::pin::Pin;

/// A `ReactiveContext` wrapper that records every interesting call.
///
/// Generic over any adapter context, so devtools work identically on GPUI,
/// Leptos, Dioxus, Xilem, Floem, and the testkit. Because `Instrumented`
/// itself implements [`ReactiveContext`], wrappers can be stacked (e.g. two
/// scopes recording into different logs).
///
/// # Example
///
/// ```rust
/// use quoin_core::prelude::*;
/// use quoin_devtools::{Instrumented, enable};
/// use quoin_testkit::TestContext;
///
/// enable(); // or set QUOIN_TRACE=1 / use a dedicated EventLog
/// let cx = Instrumented::wrap(TestContext::new(), "app");
/// let count = cx.create_signal(0u32);
/// count.set(1);
/// let summary = quoin_devtools::global_log().summary();
/// assert!(summary.values().any(|count| *count > 0));
/// quoin_devtools::disable();
/// ```
pub struct Instrumented<C> {
    inner: C,
    scope: &'static str,
    log: &'static EventLog,
}

impl<C> Clone for Instrumented<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scope: self.scope,
            log: self.log,
        }
    }
}

impl<C> Instrumented<C> {
    /// Wraps `context`, recording into `log` under `scope`.
    #[must_use]
    pub const fn wrap_in(context: C, scope: &'static str, log: &'static EventLog) -> Self {
        Self {
            inner: context,
            scope,
            log,
        }
    }

    /// Wraps `context` into the process-global log.
    #[must_use]
    pub fn wrap(context: C, scope: &'static str) -> Self {
        Self::wrap_in(context, scope, crate::global_log())
    }

    /// The wrapped context.
    #[must_use]
    pub const fn inner(&self) -> &C {
        &self.inner
    }

    /// Consumes the wrapper, returning the wrapped context.
    #[must_use]
    pub fn into_inner(self) -> C {
        self.inner
    }

    /// The scope label every event from this wrapper carries.
    #[must_use]
    pub const fn scope(&self) -> &'static str {
        self.scope
    }

    /// The log this wrapper records into.
    #[must_use]
    pub const fn log(&self) -> &'static EventLog {
        self.log
    }

    /// Builds a [`TracedSignal`] around a freshly created (or looked-up)
    /// inner signal. Single construction path for `create_signal` and
    /// `use_global`.
    fn traced_signal<S, T>(&self, inner: S) -> TracedSignal<S, T> {
        TracedSignal {
            inner,
            scope: self.scope,
            log: self.log,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: ReactiveContext> ReactiveContext for Instrumented<C> {
    type Signal<T: Clone + Send + 'static> = TracedSignal<C::Signal<T>, T>;
    type Executor = TracedExecutor<C::Executor>;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        record(
            self.log,
            EventKind::SignalCreated,
            self.scope,
            Some(std::any::type_name::<T>()),
        );
        self.traced_signal(self.inner.create_signal(initial))
    }

    fn executor(&self) -> Self::Executor {
        TracedExecutor {
            inner: self.inner.executor(),
            scope: self.scope,
            log: self.log,
        }
    }

    fn request_update(&self) {
        record(self.log, EventKind::UpdateRequested, self.scope, None);
        self.inner.request_update();
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        record(
            self.log,
            EventKind::SignalCreated,
            self.scope,
            Some(std::any::type_name::<T>()),
        );
        self.inner.provide_global(value);
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.inner
            .use_global::<T>()
            .map(|signal| self.traced_signal(signal))
    }
}

/// A signal wrapper that records mutations.
pub struct TracedSignal<S, T> {
    inner: S,
    scope: &'static str,
    log: &'static EventLog,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<S, T> Clone for TracedSignal<S, T>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scope: self.scope,
            log: self.log,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, T> TracedSignal<S, T>
where
    S: Signal<T>,
    T: Clone + 'static,
{
    /// Records a mutation event including a `Debug` snapshot of the value,
    /// then forwards the write.
    ///
    /// The enable check consults the log this signal records into, so
    /// wrappers on custom (non-global) logs honour their own flag.
    pub fn set_dbg(&self, value: T)
    where
        T: std::fmt::Debug,
    {
        if self.log.is_enabled() {
            let snapshot = format!("{value:?}");
            let event = mutation_event(self.scope, std::any::type_name::<T>(), Some(snapshot));
            self.log.push(event);
        }
        self.inner.set(value);
    }

    /// The scope label mutations are recorded under.
    #[must_use]
    pub const fn scope(&self) -> &'static str {
        self.scope
    }
}

impl<S, T> Signal<T> for TracedSignal<S, T>
where
    S: Signal<T>,
    T: Clone + 'static,
{
    fn get(&self) -> T {
        self.inner.get()
    }

    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.inner.with(f)
    }

    fn set(&self, value: T) {
        record(
            self.log,
            EventKind::SignalMutated,
            self.scope,
            Some(std::any::type_name::<T>()),
        );
        self.inner.set(value);
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        record(
            self.log,
            EventKind::SignalMutated,
            self.scope,
            Some(std::any::type_name::<T>()),
        );
        self.inner.update(f);
    }
}

/// An executor wrapper that records task spawn, completion, and abort.
///
/// The spawned future is wrapped in a drop-guard that distinguishes normal
/// completion from cancellation (abort/drop), while the join handle is
/// returned unchanged so awaiting works exactly like the underlying adapter.
pub struct TracedExecutor<E> {
    inner: E,
    scope: &'static str,
    log: &'static EventLog,
}

impl<E> Clone for TracedExecutor<E>
where
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scope: self.scope,
            log: self.log,
        }
    }
}

struct PinnedTracedFuture<F> {
    inner: Pin<Box<F>>,
    log: &'static EventLog,
    scope: &'static str,
    completed: bool,
}

impl<F: Future> Future for PinnedTracedFuture<F> {
    type Output = F::Output;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<F::Output> {
        let result = self.inner.as_mut().poll(cx);
        if result.is_ready() {
            self.completed = true;
        }
        result
    }
}

impl<F> Drop for PinnedTracedFuture<F> {
    fn drop(&mut self) {
        let kind = if self.completed {
            EventKind::TaskCompleted
        } else {
            EventKind::TaskAborted
        };
        record(self.log, kind, self.scope, None);
    }
}

impl<E: Executor> Executor for TracedExecutor<E> {
    type JoinHandle<T: Send + 'static> = E::JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        record(self.log, EventKind::TaskSpawned, self.scope, None);
        self.inner.spawn(PinnedTracedFuture {
            inner: Box::pin(future),
            log: self.log,
            scope: self.scope,
            completed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_core::JoinHandle as _;
    use quoin_core::pool::block_on;
    use quoin_testkit::{ImmediateExecutor, TestContext};

    fn fresh_log() -> &'static EventLog {
        // Leak one isolated log per call site so parallel tests never
        // observe each other's events (leaking a handful of small test logs
        // is fine and keeps the `&'static EventLog` signature).
        let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(1024)));
        log.set_enabled(true);
        log
    }

    /// Waits (bounded) for `predicate` to hold over the log's events. Used
    /// for events recorded asynchronously by worker threads.
    fn wait_for(log: &EventLog, kind: EventKind) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if log.events().iter().any(|event| event.kind == kind) {
                return true;
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    fn disabled_by_default_no_events() {
        let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(64)));
        let cx = Instrumented::wrap_in(TestContext::new(), "t", log);
        let s = cx.create_signal(1u32);
        s.set(2);
        assert!(log.is_empty(), "disabled instrumentation must not record");
    }

    #[test]
    fn records_signal_lifecycle() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "app", log);
        let s = cx.create_signal(0u32);
        s.set(5);
        s.update(|v| *v += 1);
        assert_eq!(s.get(), 6);
        let events = log.events();
        assert!(events.iter().any(|e| e.kind == EventKind::SignalCreated));
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalMutated)
                .count(),
            2
        );
        assert!(events.iter().all(|e| e.scope == "app"));
    }

    #[test]
    fn records_updates_and_spawns() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(quoin_testkit::TestContext::new(), "spawn-scope", log);
        cx.request_update();
        let executor = cx.executor();
        let handle = executor.spawn(async { 1u32 });
        assert_eq!(block_on(handle).unwrap(), 1);
        let events = log.events();
        assert!(events.iter().any(|e| e.kind == EventKind::UpdateRequested));
        assert!(events.iter().any(|e| e.kind == EventKind::TaskSpawned));
        assert!(events.iter().any(|e| e.kind == EventKind::TaskCompleted));
    }

    #[test]
    fn set_dbg_captures_value() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "dbg", log);
        let s = cx.create_signal(String::from("hello"));
        s.set_dbg(String::from("world"));
        assert!(
            log.events()
                .iter()
                .any(|e| e.value.as_deref() == Some("\"world\""))
        );
    }

    /// Regression: `set_dbg` must consult the log the signal records into,
    /// not the process-global log.
    #[test]
    fn set_dbg_honours_custom_log_flag() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "custom", log);
        let s = cx.create_signal(1u32);
        s.set_dbg(2u32);
        assert!(
            log.events().iter().any(|e| e.value.as_deref() == Some("2")),
            "custom log with its own enable flag must record set_dbg"
        );
    }

    #[test]
    fn hot_signals_ranks_mutations() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "hot", log);
        let a = cx.create_signal(0u32);
        let b = cx.create_signal(0i64);
        for _ in 0..5 {
            a.set(1);
        }
        b.set(2);
        let hot = log.hot_signals();
        assert_eq!(hot.first().map(|(n, _)| *n), Some("u32"));
        assert_eq!(hot.first().map(|(_, c)| *c), Some(5));
    }

    #[test]
    fn summary_counts_per_scope() {
        let log = fresh_log();
        let a = Instrumented::wrap_in(TestContext::new(), "a", log);
        let b = Instrumented::wrap_in(TestContext::new(), "b", log);
        a.create_signal(0u32).set(1);
        b.create_signal(0u32).set(1);
        let summary = log.summary();
        assert_eq!(summary.get(&("a", EventKind::SignalMutated)), Some(&1));
        assert_eq!(summary.get(&("b", EventKind::SignalMutated)), Some(&1));
    }

    #[test]
    fn aborting_task_records_abort_event() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(
            TestContext::with_executor(quoin_testkit::ThreadedExecutor),
            "abort",
            log,
        );
        let executor = cx.executor();
        {
            let handle = executor.spawn(async {
                // Never completes on its own; dropped while pending.
                std::future::pending::<()>().await;
            });
            handle.abort();
        }
        // The drop-guard fires on the worker thread, so poll for it.
        assert!(
            wait_for(log, EventKind::TaskAborted),
            "aborting a pending task must eventually record TaskAborted"
        );
        assert!(
            log.events()
                .iter()
                .any(|e| e.kind == EventKind::TaskSpawned)
        );
    }

    #[test]
    fn instrumented_exposes_scope_and_log() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "meta", log);
        assert_eq!(cx.scope(), "meta");
        assert!(std::ptr::eq(cx.log(), log));
        let inner: TestContext = cx.into_inner();
        let _ = inner.create_signal(0u32); // unwrapped: no more recording
        assert_eq!(log.events().iter().filter(|e| e.scope == "meta").count(), 0);
    }

    /// `Instrumented` is itself a `ReactiveContext`, so wrappers stack:
    /// an outer scope and an inner scope can record into different logs.
    #[test]
    fn instrumented_wrappers_can_nest() {
        let outer_log = fresh_log();
        let inner_log = fresh_log();
        let outer = Instrumented::wrap_in(TestContext::new(), "outer", outer_log);
        let inner = Instrumented::wrap_in(outer, "inner", inner_log);
        let s = inner.create_signal(0u32);
        s.set(1);
        let inner_events = inner_log.events();
        assert_eq!(inner_events.len(), 2, "inner wrapper records its own calls");
        assert!(inner_events.iter().all(|e| e.scope == "inner"));
        // The outer wrapper sees the forwarded create + set through its own
        // instrumentation layer as well.
        let outer_events = outer_log.events();
        assert!(
            outer_events
                .iter()
                .any(|e| e.kind == EventKind::SignalCreated && e.scope == "outer")
        );
    }

    #[test]
    fn use_global_records_creation() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(TestContext::new(), "globals", log);
        cx.provide_global(10u32);
        let signal = cx.use_global::<u32>().expect("global present");
        signal.set(11);
        let events = log.events();
        assert!(events.iter().any(|e| e.kind == EventKind::SignalCreated));
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalMutated)
                .count(),
            1
        );
    }

    #[test]
    fn immediate_executor_records_completed_tasks() {
        let log = fresh_log();
        let cx = Instrumented::wrap_in(
            TestContext::with_executor(ImmediateExecutor),
            "immediate",
            log,
        );
        let executor = cx.executor();
        let handle = executor.spawn(async { 21 * 2 });
        assert_eq!(handle.take_result(), Some(42));
        let events = log.events();
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::TaskCompleted)
                .count(),
            1,
            "inline-completed futures record TaskCompleted on drop"
        );
    }
}
