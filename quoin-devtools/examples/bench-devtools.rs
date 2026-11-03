//! Micro-benchmarks for the quoin-devtools event log.
//!
//! No benchmark framework — plain `std::time::Instant` so this runs anywhere:
//!
//! ```bash
//! cargo run --release -p quoin-devtools --example bench-devtools
//! ```
//!
//! Numbers are machine-relative; treat them as regression baselines to
//! compare across commits on the same machine, not absolute truth.

use quoin_core::prelude::*;
use quoin_devtools::{Event, EventKind, EventLog, Instrumented};
use quoin_testkit::{ImmediateExecutor, TestContext};
use std::time::Instant;

fn bench<F: FnMut()>(name: &str, iterations: u32, mut f: F) {
    // Warmup.
    f();
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_secs_f64() * 1e9 / f64::from(iterations);
    println!("{name:<52} {iterations:>9} ops in {elapsed:>10.2?}  ({ns_per_op:>9.0} ns/op)");
}

fn main() {
    println!("quoin-devtools micro-benchmarks (release build)\n");

    // --- Recording (enabled vs disabled) ------------------------------------
    let enabled_log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(64)));
    enabled_log.set_enabled(true);
    bench("record (enabled, small ring)", 1_000_000, || {
        quoin_devtools::record(enabled_log, EventKind::SignalMutated, "bench", None);
    });

    let disabled_log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(64)));
    bench("record (disabled: atomic load only)", 1_000_000, || {
        quoin_devtools::record(disabled_log, EventKind::SignalMutated, "bench", None);
    });

    // --- Instrumented wrapper overhead --------------------------------------
    let raw = TestContext::with_executor(ImmediateExecutor);
    let signal = raw.create_signal(0u32);
    bench("raw set (no wrapper)", 1_000_000, || signal.set(1));

    let cx = Instrumented::wrap(TestContext::with_executor(ImmediateExecutor), "bench");
    let traced = cx.create_signal(0u32);
    bench("Instrumented set (recording off)", 1_000_000, || {
        traced.set(1);
    });

    let log = cx.log();
    log.set_enabled(true);
    bench("Instrumented set (recording on)", 1_000_000, || {
        traced.set(1);
    });
    log.set_enabled(false);

    // --- Aggregate views over a full ring -----------------------------------
    let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(4_096)));
    log.set_enabled(true);
    for i in 0..4_096u32 {
        let kind = match i % 3 {
            0 => EventKind::SignalMutated,
            1 => EventKind::UpdateRequested,
            _ => EventKind::TaskSpawned,
        };
        quoin_devtools::record(log, kind, "view", Some("u32"));
    }

    bench("summary() over 4096 events", 10_000, || {
        let _ = log.summary();
    });
    bench("hot_signals() over 4096 events", 10_000, || {
        let _ = log.hot_signals();
    });
    bench("len() (single lock)", 1_000_000, || {
        let _ = log.len();
    });
    bench("with_events(count) (zero-copy view)", 100_000, || {
        let _ = log.with_events(<[Event]>::len);
    });

    println!("\nbaseline recorded — compare across commits on the same machine.");
}
