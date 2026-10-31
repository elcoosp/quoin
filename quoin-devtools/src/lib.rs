//! # quoin-devtools — see what your reactive app is doing
//!
//! A framework-agnostic instrumentation layer for quoin. Wrap **any**
//! adapter's `ReactiveContext` (GPUI, Leptos, Dioxus, Xilem, Floem, or the
//! `quoin-testkit` context) and every signal creation, read-triggering
//! mutation, task spawn, and UI update request is recorded into an
//! inspectable [`EventLog`].
//!
//! ```text
//! use quoin_devtools::Instrumented;
//!
//! let raw_ctx = /* any adapter context */;
//! let ctx = Instrumented::wrap(raw_ctx, "app");
//! // use `ctx` everywhere you used `raw_ctx`
//! ```
//!
//! ## Reading the log
//!
//! ```text
//! quoin_devtools::global_log().summary().print(); // stderr table
//! for event in quoin_devtools::global_log().events() { … }
//! ```
//!
//! ## Zero-cost when disabled
//!
//! Unless [`enabled()`] is turned on (default: the `QUOIN_TRACE=1`/`QUOIN_TRACE=on`
//! environment variable, or an explicit `init()` call), the `Instrumented`
//! wrapper short-circuits every recording call behind one relaxed atomic
//! load, and no events are allocated.

use quoin_core::executor::Executor;
use quoin_core::reactive::ReactiveContext;
use quoin_core::signal::Signal;
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// How many events the default log keeps before overwriting the oldest.
pub const DEFAULT_LOG_CAPACITY: usize = 4096;

static GLOBAL_LOG: OnceLock<EventLog> = OnceLock::new();

/// Returns `true` when the global log is recording.
#[must_use]
pub fn enabled() -> bool {
    global_log().is_enabled()
}

/// Turns instrumentation on for the global log.
pub fn enable() {
    global_log().set_enabled(true);
}

/// Turns instrumentation off for the global log.
pub fn disable() {
    global_log().set_enabled(false);
}

/// Enables instrumentation when the `QUOIN_TRACE` environment variable is
/// set to a truthy value (`1`, `on`, `true`, `yes`).
///
/// Call this once at the top of your `main` / app bootstrap. It is the
/// default behaviour of [`Instrumented::wrap`].
pub fn init_from_env() {
    if let Ok(value) = std::env::var("QUOIN_TRACE")
        && parse_trace_flag(&value)
    {
        enable();
    }
}

/// Truthiness rule for the `QUOIN_TRACE` flag (exposed for testing).
#[must_use]
pub fn parse_trace_flag(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "on" | "true" | "yes"
    )
}

/// The process-wide default [`EventLog`].
#[must_use]
pub fn global_log() -> &'static EventLog {
    GLOBAL_LOG.get_or_init(|| EventLog::with_capacity(DEFAULT_LOG_CAPACITY))
}

/// The kind of recorded event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKind {
    /// A signal was created.
    SignalCreated,
    /// A signal was written (`set` or `update`).
    SignalMutated,
    /// `request_update` was called.
    UpdateRequested,
    /// An async task was spawned.
    TaskSpawned,
    /// A running task was aborted.
    TaskAborted,
    /// A task ran to completion.
    TaskCompleted,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl EventKind {
    /// Short human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            EventKind::SignalCreated => "created",
            EventKind::SignalMutated => "mutated",
            EventKind::UpdateRequested => "update",
            EventKind::TaskSpawned => "spawn",
            EventKind::TaskAborted => "abort",
            EventKind::TaskCompleted => "done",
        }
    }
}

/// One recorded trace event.
#[derive(Debug, Clone)]
pub struct Event {
    /// Monotonic sequence number within the log.
    pub seq: u64,
    /// When the event happened (relative to log creation).
    pub at: Instant,
    /// What happened.
    pub kind: EventKind,
    /// The instrumented scope label (e.g. `"app"`, `"search-panel"`).
    pub scope: &'static str,
    /// Rust type name of the signal payload, when applicable.
    pub type_name: Option<&'static str>,
    /// Debug snapshot of the new value, when captured via
    /// [`snapshot`](fn@snapshot) or `TracedSignal::set_dbg`.
    pub value: Option<String>,
    /// Thread that produced the event.
    pub thread: String,
}

impl Event {
    fn now(
        seq: u64,
        kind: EventKind,
        scope: &'static str,
        type_name: Option<&'static str>,
    ) -> Self {
        Self {
            seq,
            at: Instant::now(),
            kind,
            scope,
            type_name,
            value: None,
            thread: std::thread::current()
                .name()
                .unwrap_or("<unnamed>")
                .to_string(),
        }
    }
}

/// Bounded, thread-safe ring of [`Event`]s with aggregate views.
///
/// Each log owns its own enable flag, so different parts of an app (or
/// different tests) can record into independent logs without interfering.
#[derive(Debug)]
pub struct EventLog {
    events: Mutex<VecDequeRing>,
    seq: AtomicU64,
    started: Instant,
    enabled: AtomicBool,
}

#[derive(Debug)]
struct VecDequeRing {
    buffer: std::collections::VecDeque<Event>,
    capacity: usize,
}

impl EventLog {
    /// Creates a log holding up to `capacity` events (oldest dropped first).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Mutex::new(VecDequeRing {
                buffer: std::collections::VecDeque::with_capacity(capacity.min(1024)),
                capacity,
            }),
            seq: AtomicU64::new(0),
            started: Instant::now(),
            enabled: AtomicBool::new(false),
        }
    }

    /// Whether this log records events.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Turns recording on or off for this log.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    fn push(&self, mut event: Event) {
        event.seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let mut ring = self.events.lock().unwrap();
        if ring.buffer.len() >= ring.capacity {
            ring.buffer.pop_front();
        }
        ring.buffer.push_back(event);
    }

    /// Snapshot of the currently retained events (oldest first).
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().buffer.iter().cloned().collect()
    }

    /// Number of retained events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.lock().unwrap().buffer.len()
    }

    /// Whether no events are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drops all retained events (sequence numbers keep increasing).
    pub fn clear(&self) {
        self.events.lock().unwrap().buffer.clear();
    }

    /// Per-`(scope, kind)` event counts over the retained window.
    #[must_use]
    pub fn summary(&self) -> BTreeMap<(&'static str, EventKind), usize> {
        let mut map = BTreeMap::new();
        for event in self.events() {
            *map.entry((event.scope, event.kind)).or_insert(0) += 1;
        }
        map
    }

    /// Per-signal-type mutation counts (the "hot signal" view).
    #[must_use]
    pub fn hot_signals(&self) -> Vec<(&'static str, usize)> {
        let mut map: BTreeMap<&'static str, usize> = BTreeMap::new();
        for event in self.events() {
            if event.kind == EventKind::SignalMutated
                && let Some(name) = event.type_name
            {
                *map.entry(name).or_insert(0) += 1;
            }
        }
        let mut v: Vec<_> = map.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        v
    }

    /// Wall-clock age of the log.
    #[must_use]
    pub fn uptime(&self) -> std::time::Duration {
        self.started.elapsed()
    }
}

impl EventLog {
    /// Prints a compact human-readable table to stderr.
    pub fn dump(&self) {
        eprintln!("== quoin-devtools ({} events retained) ==", self.len());
        for ((scope, kind), count) in self.summary() {
            eprintln!("  {scope}/{kind:<60} {count:>6}");
        }
        let hot = self.hot_signals();
        if !hot.is_empty() {
            eprintln!("hot signals (mutations by payload type):");
            for (name, count) in hot.iter().take(10) {
                eprintln!("  {name:<50} {count:>6}");
            }
        }
    }
}

/// Records an event into `log` if that log is enabled.
pub fn record(
    log: &EventLog,
    kind: EventKind,
    scope: &'static str,
    type_name: Option<&'static str>,
) {
    if log.is_enabled() {
        log.push(Event::now(0, kind, scope, type_name));
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "quoin_devtools", kind = kind.label(), scope, ?type_name);
    }
}

/// Convenience: records a mutation event with a debug value snapshot.
///
/// Works with any signal value that implements `Debug` — call it from hot
/// paths where seeing the actual value matters:
///
/// ```
/// use quoin_core::prelude::*;
/// use quoin_devtools::{snapshot, global_log, enable};
///
/// enable();
/// let cx = quoin_testkit::TestContext::new();
/// let count = cx.create_signal(0u32);
/// count.set(3);
/// snapshot(&count, "app"); // records SignalMutated with value "3"
/// let events = global_log().events();
/// assert!(events.iter().any(|e| e.value.as_deref() == Some("3")));
/// ```
pub fn snapshot<S, T>(signal: &S, scope: &'static str)
where
    S: Signal<T>,
    T: std::fmt::Debug + Clone + 'static,
{
    let log = global_log();
    if !log.is_enabled() {
        return;
    }
    let mut event = Event::now(
        0,
        EventKind::SignalMutated,
        scope,
        Some(std::any::type_name::<T>()),
    );
    event.value = Some(format!("{:?}", signal.get()));
    log.push(event);
}

// ---------------------------------------------------------------------------
// Instrumented context
// ---------------------------------------------------------------------------

/// A `ReactiveContext` wrapper that records every interesting call.
///
/// Generic over any adapter context, so devtools work identically on GPUI,
/// Leptos, Dioxus, Xilem, Floem, and the testkit.
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
    pub fn wrap_in(context: C, scope: &'static str, log: &'static EventLog) -> Self {
        Self {
            inner: context,
            scope,
            log,
        }
    }

    /// Wraps `context` into the process-global log.
    #[must_use]
    pub fn wrap(context: C, scope: &'static str) -> Self {
        Self::wrap_in(context, scope, global_log())
    }

    /// The wrapped context.
    #[must_use]
    pub fn inner(&self) -> &C {
        &self.inner
    }
}

impl<C: ReactiveContext> ReactiveContext for Instrumented<C> {
    type Signal<T: Clone + 'static> = TracedSignal<C::Signal<T>, T>;
    type Executor = TracedExecutor<C::Executor>;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T> {
        record(
            self.log,
            EventKind::SignalCreated,
            self.scope,
            Some(std::any::type_name::<T>()),
        );
        TracedSignal {
            inner: self.inner.create_signal(initial),
            scope: self.scope,
            log: self.log,
            _marker: std::marker::PhantomData,
        }
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
        self.inner.use_global::<T>().map(|signal| TracedSignal {
            inner: signal,
            scope: self.scope,
            log: self.log,
            _marker: std::marker::PhantomData,
        })
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
    /// Records a mutation event including a `Debug` snapshot of the value.
    pub fn set_dbg(&self, value: T)
    where
        T: std::fmt::Debug,
    {
        if enabled() {
            let mut event = Event::now(
                0,
                EventKind::SignalMutated,
                self.scope,
                Some(std::any::type_name::<T>()),
            );
            event.value = Some(format!("{value:?}"));
            self.log.push(event);
        }
        self.inner.set(value);
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
    use quoin_testkit::TestContext;

    fn fresh_log() -> &'static EventLog {
        // Leak one isolated log per call site so parallel tests never
        // observe each other's events (leaking a handful of small test logs
        // is fine and keeps the `&'static EventLog` signature).
        let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(1024)));
        log.set_enabled(true);
        log
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
        enable();
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
        enable();
        let log = fresh_log();
        let cx = Instrumented::wrap_in(quoin_testkit::TestContext::new(), "spawn-scope", log);
        cx.request_update();
        let executor = cx.executor();
        let handle = executor.spawn(async { 1u32 });
        assert_eq!(quoin_core::pool::block_on(handle).unwrap(), 1);
        let events = log.events();
        assert!(events.iter().any(|e| e.kind == EventKind::UpdateRequested));
        assert!(events.iter().any(|e| e.kind == EventKind::TaskSpawned));
    }

    #[test]
    fn set_dbg_captures_value() {
        enable();
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

    #[test]
    fn hot_signals_ranks_mutations() {
        enable();
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
        enable();
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
    fn parse_trace_flag_truthiness() {
        assert!(parse_trace_flag("1"));
        assert!(parse_trace_flag("on"));
        assert!(parse_trace_flag("TRUE"));
        assert!(parse_trace_flag("Yes"));
        assert!(!parse_trace_flag(""));
        assert!(!parse_trace_flag("0"));
        assert!(!parse_trace_flag("off"));
        assert!(!parse_trace_flag("nope"));
    }
}
