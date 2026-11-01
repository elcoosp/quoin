//! Visual counter demo with the quoin devtools wired in.
//!
//! The `XilemContext` is wrapped in an [`Instrumented`] context: every signal
//! creation, mutation, and update request this window performs is recorded
//! into the global [`EventLog`], and the window shows the live trace counts
//! right next to the counter — the "debug the whole dev loop" story inside an
//! actual GUI. When the window closes, the full event table is dumped to
//! stderr.
//!
//! ```bash
//! cargo run --manifest-path examples/counter-xilem/Cargo.toml
//! # tracing stays on in this demo; QUOIN_TRACE=0 documents the intent
//! ```
//!
//! The same [`Instrumented`] wrapper works unchanged with GPUI/Leptos/Dioxus/
//! Floem contexts, because it wraps the `ReactiveContext` trait — see
//! `examples/devtools-demo` for the headless variant.

use counter_lib::use_counter_send;
use quoin::devtools::{EventKind, Instrumented, TracedSignal, enable, global_log, init_from_env};
use quoin::prelude::*;
use std::sync::Arc;
use xilem::tokio::runtime::Runtime;
use xilem::{
    EventLoop, WindowOptions, Xilem,
    view::{flex_col, label, text_button},
};

/// The trace scope every event from this window is recorded under.
const SCOPE: &str = "counter-xilem";

/// The shared counter hook as produced by an `Instrumented` Xilem context:
/// a `TracedSignal` count plus a `Send` increment action.
type InstrumentedCounter =
    counter_lib::Counter<TracedSignal<XilemSignal<u32>, u32>, Arc<dyn Fn() + Send + Sync>>;

/// The application state: just the shared counter hook, created from an
/// `Instrumented` context. The signals keep the underlying context alive.
struct CounterApp {
    counter: InstrumentedCounter,
}

/// One line summarizing the trace activity recorded for this window; the
/// label re-renders together with the count, so the panel stays live.
fn trace_summary_line() -> String {
    let counts = global_log().summary();
    let count = |kind| counts.get(&(SCOPE, kind)).copied().unwrap_or(0);
    format!(
        "devtools: {} created, {} mutated, {} update requests",
        count(EventKind::SignalCreated),
        count(EventKind::SignalMutated),
        count(EventKind::UpdateRequested),
    )
}

fn main() -> Result<(), winit::error::EventLoopError> {
    // QUOIN_TRACE=1 also works; the demo turns recording on unconditionally
    // so the trace panel is always live.
    init_from_env();
    enable();

    let runtime = Arc::new(Runtime::new().expect("failed to start tokio runtime"));
    let ctx = Instrumented::wrap(XilemContext::new(runtime), SCOPE);
    ctx.inner().set_update_notifier(|| {
        // Xilem rebuilds its view after button actions; the notifier exists
        // for adapters/timings where a manual rebuild request is required.
    });

    let app = CounterApp {
        counter: use_counter_send(&ctx),
    };

    let result = Xilem::new_simple(
        app,
        |state: &mut CounterApp| {
            let count = state.counter.count.get();
            let trace = trace_summary_line();
            flex_col((
                label(format!("Count: {count}")),
                text_button("Increment", |state: &mut CounterApp| {
                    (state.counter.increment)();
                }),
                text_button("Reset", |state: &mut CounterApp| {
                    state.counter.count.set(0);
                }),
                label(trace),
            ))
        },
        WindowOptions::new("Counter - quoin + devtools"),
    )
    .run_in(EventLoop::with_user_event());

    // Window closed: print the recorded session.
    let log = global_log();
    log.dump();
    println!("\nevents ({} retained):", log.len());
    for event in log.events().iter().take(50) {
        println!(
            "  [{:>7}] scope={:<14} type={:<24} {}",
            event.kind,
            event.scope,
            event.type_name.unwrap_or("-"),
            match &event.value {
                Some(value) => format!("value={value}"),
                None => String::new(),
            }
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin::devtools::EventLog;

    /// Runs a headless session against `cx`: simulates `clicks` button
    /// presses and returns the final count, exercising exactly the code path
    /// the window uses (shared counter hook on the instrumented context).
    fn run_session<C>(cx: &C, clicks: usize) -> u32
    where
        C: ReactiveContext,
        C::Signal<u32>: Send + Sync + 'static,
    {
        let counter = use_counter_send(cx);
        for _ in 0..clicks {
            (counter.increment)();
        }
        counter.count.get()
    }

    fn session_log() -> &'static EventLog {
        // One isolated log per call site keeps parallel tests independent.
        let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(256)));
        log.set_enabled(true);
        log
    }

    #[test]
    fn instrumented_counter_session_records_trace() {
        let log = session_log();
        let runtime = Arc::new(Runtime::new().expect("tokio runtime"));
        let cx = Instrumented::wrap_in(XilemContext::new(runtime), "test-session", log);

        assert_eq!(run_session(&cx, 3), 3);

        let events = log.events();
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalCreated)
                .count(),
            1,
            "one signal is created by the counter hook"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalMutated)
                .count(),
            3,
            "each simulated click records one mutation"
        );
        assert!(events.iter().all(|e| e.scope == "test-session"));
    }

    #[test]
    fn trace_summary_line_counts_this_scope() {
        let log = session_log();
        let runtime = Arc::new(Runtime::new().expect("tokio runtime"));
        let cx = Instrumented::wrap_in(XilemContext::new(runtime), "summary-scope", log);
        let counter = use_counter_send(&cx);
        (counter.increment)();

        // The demo's summary helper reads the global log, so mirror its
        // lookup for the same scope to prove the counting logic.
        let counts = log.summary();
        let mutated = counts
            .get(&("summary-scope", EventKind::SignalMutated))
            .copied()
            .unwrap_or(0);
        assert_eq!(mutated, 1);
        assert_eq!(counter.count.get(), 1);
    }

    #[test]
    fn reset_writes_zero_through_traced_signal() {
        let log = session_log();
        let runtime = Arc::new(Runtime::new().expect("tokio runtime"));
        let cx = Instrumented::wrap_in(XilemContext::new(runtime), "reset-scope", log);
        let app_counter = use_counter_send(&cx);
        for _ in 0..5 {
            (app_counter.increment)();
        }
        app_counter.count.set(0);
        assert_eq!(app_counter.count.get(), 0);
        assert!(
            log.events()
                .iter()
                .filter(|e| e.scope == "reset-scope" && e.kind == EventKind::SignalMutated)
                .count()
                >= 6,
            "five increments plus the reset are all recorded"
        );
    }
}
