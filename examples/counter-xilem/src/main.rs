//! Visual counter demo — the same `quoin_render!` source the GPUI, Leptos,
//! Dioxus and Floem demos use, rendered through the Xilem backend, with the
//! rebuild pump installed so background signal writes repaint the window.
//!
//! The `XilemContext` is wrapped in an [`Instrumented`] context: every signal
//! creation, mutation, and update request this window performs is recorded
//! into the global [`EventLog`].
//!
//! ```bash
//! cargo run --manifest-path examples/counter-xilem/Cargo.toml
//! ```

use counter_lib::use_counter_send;
use quoin::devtools::{EventKind, Instrumented, TracedSignal, enable, global_log, init_from_env};
use quoin::prelude::*;
use quoin::xilem_pump;
use std::sync::Arc;
use xilem::tokio::runtime::Runtime;
use xilem::{EventLoop, WindowOptions, Xilem};

/// The trace scope every event from this window is recorded under.
const SCOPE: &str = "counter-xilem";

/// The shared counter hook as produced by an `Instrumented` Xilem context.
type InstrumentedCounter =
    counter_lib::Counter<TracedSignal<XilemSignal<u32>, u32>, Arc<dyn Fn() + Send + Sync>>;

/// The application state: just the shared counter hook.
struct CounterApp {
    counter: InstrumentedCounter,
}

/// One line summarizing the trace activity recorded for this window.
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
    init_from_env();
    enable();

    let runtime = Arc::new(Runtime::new().expect("failed to start tokio runtime"));
    let ctx = Instrumented::wrap(XilemContext::new(runtime), SCOPE);
    // Background signal writes request rebuilds through the pump.
    xilem_pump::install_notifier(ctx.inner());

    let app = CounterApp {
        counter: use_counter_send(&ctx),
    };

    let result = Xilem::new_simple(
        app,
        |state: &mut CounterApp| {
            let count = state.counter.count.clone();
            let trace = trace_summary_line();

            // The pump view rides along in the tree; the counter UI is the
            // same quoin_render! source the other backends render.
            xilem::view::flex(
                xilem::kurbo::Axis::Vertical,
                (xilem_pump::pump_view(), quoin_render! {
                    div(class: "flex flex-col gap-2 p-4") {
                        h1() { "Counter" }
                        p() { { format!("Count: {}", count.get()) } }
                        button(label: "Increment",
                               on_click: |_| count.update(|c| *c += 1))
                        button(label: "Reset",
                               on_click: |_| count.set(0))
                        p(class: "text-muted-foreground text-sm") { { trace } }
                    }
                }),
            )
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
    use quoin::ReactiveContext;

    /// Runs a headless session against `cx`: simulates `clicks` button
    /// presses and returns the final count.
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
    fn background_write_requests_pump_rebuild() {
        // A write from a non-main thread flips the pump's dirty flag; with a
        // pump view installed the driver would rebuild, and the flag clears.
        let runtime = Arc::new(Runtime::new().expect("tokio runtime"));
        let ctx = XilemContext::new(runtime);
        xilem_pump::install_notifier(&ctx);
        let counter = use_counter_send(&ctx);

        let handle = std::thread::spawn(move || {
            counter.count.set(42);
        });
        handle.join().expect("background write");

        assert!(
            xilem_pump::is_dirty(),
            "the background write must request a rebuild through the pump"
        );
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
