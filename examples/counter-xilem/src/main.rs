//! Visual counter demo — the same `quoin_render!` source the GPUI, Leptos,
//! Dioxus and Floem demos use, rendered through the Xilem backend via
//! `component!` + `run_app!`.
//!
//! The expansion owns the state struct, creates the signals, installs the
//! rebuild pump (background signal writes repaint the window), and boots the
//! app — this file contains no Xilem boilerplate at all.
//!
//! For the devtools-instrumented variant of the same demo see
//! `src/bin/traced.rs`: tracing needs the `Instrumented` context wrapper,
//! which is the documented hand-written escape hatch when a component needs
//! more than `component!` expresses.
//!
//! ```bash
//! cargo run --manifest-path examples/counter-xilem/Cargo.toml
//! cargo run --manifest-path examples/counter-xilem/Cargo.toml --bin traced
//! ```

use counter_lib::use_counter_send;
use quoin::devtools::{EventKind, Instrumented, TracedSignal, global_log, init_from_env};
use quoin::prelude::*;
use std::sync::Arc;
use xilem::tokio::runtime::Runtime;
use quoin::XilemContext;

component! {
    pub CounterApp {
        state { count: u32 = 0 }

        render {
            quoin_render! {
                div(class: "flex flex-col gap-2 p-4") {
                    h1() { "Counter" }
                    p() { { format!("Count: {}", count.get()) } }
                    button(label: "Increment",
                           on_click: |_| count.update(|c| *c += 1))
                    button(label: "Reset",
                           on_click: |_| count.set(0))
                }
            }
        }
    }
}

fn main() {
    run_app!(
        CounterApp,
        window_opts: ::xilem::WindowOptions::new("Counter - quoin")
    );
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

    /// The `component!` state struct constructs from a context: its signals
    /// are created through the standard `ReactiveContext` path.
    #[test]
    fn component_constructs_and_starts_at_zero() {
        let runtime = Arc::new(Runtime::new().expect("tokio runtime"));
        let ctx = XilemContext::new(runtime);
        let app = CounterApp::new(&ctx);
        assert_eq!(app.count.get(), 0);
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
        quoin::xilem_pump::install_notifier(&ctx);
        let counter = use_counter_send(&ctx);

        let handle = std::thread::spawn(move || {
            counter.count.set(42);
        });
        handle.join().expect("background write");

        assert!(
            quoin::xilem_pump::is_dirty(),
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
