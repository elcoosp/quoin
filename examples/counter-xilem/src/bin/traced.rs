//! Devtools-instrumented counter — the hand-written escape hatch twin of
//! `main.rs`.
//!
//! `main.rs` builds the same counter with `component!` + `run_app!`. This
//! binary needs the `Instrumented` context wrapper (every signal creation,
//! mutation, and update request is recorded into the global [`EventLog`]),
//! which does not fit the `component!` shape — so it keeps the explicit
//! state struct and `Xilem::new_simple` form. That is the documented path
//! for apps that need more than `component!` expresses.
//!
//! ```bash
//! cargo run --manifest-path examples/counter-xilem/Cargo.toml --bin traced
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
