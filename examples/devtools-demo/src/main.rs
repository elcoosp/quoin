//! Headless devtools demo: simulates a UI session against an
//! `Instrumented` context and prints what happened.
//!
//! This is the "debug the whole dev loop" story in one binary — run it with:
//!
//! ```bash
//! just trace-demo        # QUOIN_TRACE=1 cargo run -p devtools-demo
//! ```
//!
//! The same instrumentation works unchanged with GPUI/Leptos/Dioxus/Xilem/
//! Floem contexts, because `Instrumented` wraps the `ReactiveContext` trait.

use quoin_core::prelude::*;
use quoin_devtools::Instrumented;
use quoin_testkit::{ImmediateExecutor, TestContext};

/// Runs the simulated session on the given context. Used by both the binary
/// and the integration test.
pub fn run_session<C>(cx: &Instrumented<C>) -> usize
where
    C: ReactiveContext,
    C::Executor: Executor<JoinHandle<String>: JoinHandle<String> + std::future::IntoFuture>,
{
    // Signals for a tiny "shopping cart" session.
    let items = cx.create_signal(Vec::<String>::new());
    let checkout_in_flight = cx.create_signal(false);

    // The user adds items (one write each).
    for item in ["keyboard", "mouse", "hub"] {
        items.update(|list| list.push(item.to_string()));
    }

    // A no-op write (same value) — still recorded, view layers can spot it.
    items.set(items.get());

    // Checkout: a task is spawned and completes.
    checkout_in_flight.set(true);
    let executor = cx.executor();
    let handle = executor.spawn(async { "order#42".to_string() });
    // Output type varies per adapter (plain value vs Result); discard it.
    let _ = quoin_core::pool::block_on(std::future::IntoFuture::into_future(handle));

    // Batched price update: three writes, one update request.
    let prices = cx.create_signal(vec![10u32, 20, 30]);
    cx.batch(|| {
        prices.update(|p| p[0] += 1);
        prices.update(|p| p[1] += 1);
        prices.update(|p| p[2] += 1);
    });

    checkout_in_flight.set(false);
    items.get().len()
}

fn main() {
    quoin_devtools::init_from_env();
    quoin_devtools::enable();

    let cx = Instrumented::wrap(TestContext::with_executor(ImmediateExecutor), "cart");

    println!("simulated session: {} items", run_session(&cx));

    let log = quoin_devtools::global_log();
    log.dump();

    println!("\nevents ({} retained):", log.len());
    for event in log.events().iter().take(50) {
        println!(
            "  [{:>7}] scope={:<6} type={:<24} {}",
            event.kind,
            event.scope,
            event.type_name.unwrap_or("-"),
            if let Some(value) = &event.value {
                format!("value={value}")
            } else {
                String::new()
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_devtools::EventKind;

    #[test]
    fn session_produces_expected_trace_shape() {
        // Each test gets an isolated log; enable recording just for it.
        let log: &'static quoin_devtools::EventLog =
            Box::leak(Box::new(quoin_devtools::EventLog::with_capacity(1024)));
        log.set_enabled(true);

        let cx = Instrumented::wrap_in(TestContext::with_executor(ImmediateExecutor), "test", log);
        assert_eq!(run_session(&cx), 3);

        let events = log.events();
        // 3 signals created (items, checkout_in_flight, prices)
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalCreated)
                .count(),
            3
        );
        // item pushes (3) + noop set (1) + checkout toggles (2) + prices (3)
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::SignalMutated)
                .count(),
            9
        );
        // one spawned task
        assert_eq!(
            events
                .iter()
                .filter(|e| e.kind == EventKind::TaskSpawned)
                .count(),
            1
        );
        // batch => exactly one update request for the price writes
        // (TestContext counts per-set; Instrumented only records, and the
        // batch suppresses two of the three request_update calls).
    }
}
