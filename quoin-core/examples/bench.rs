//! Micro-benchmarks for the quoin core primitives.
//!
//! No benchmark framework — plain `std::time::Instant` so this runs anywhere:
//!
//! ```bash
//! just bench-core          # cargo run --release -p quoin-core --example bench
//! ```
//!
//! Numbers are machine-relative; treat them as regression baselines to
//! compare across commits on the same machine, not absolute truth.

use quoin_core::prelude::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn bench<F: FnMut()>(name: &str, iterations: usize, mut f: F) {
    // Warmup.
    f();
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!(
        "{name:<46} {iterations:>9} ops in {elapsed:>10.2?}  ({:>9.0} ns/op)",
        elapsed.as_nanos() as f64 / iterations as f64
    );
}

fn main() {
    println!("quoin-core micro-benchmarks (release build)\n");

    // --- Signals -----------------------------------------------------------
    let cx = quoin_core::test_support::StaticContext;
    let counter = Arc::new(AtomicUsize::new(0));

    let signal = cx.create_signal(0u32);
    bench("signal.set", 1_000_000, || signal.set(1));

    let signal = cx.create_signal(0u32);
    bench("signal.set (batched x10)", 100_000, || {
        cx.batch(|| {
            for _ in 0..10 {
                signal.set(1);
            }
        })
    });

    let signal = cx.create_signal(1u32);
    bench("signal.get (clone u32)", 1_000_000, || {
        let _ = signal.get();
    });

    let signal = cx.create_signal(1u32);
    bench("signal.with (no clone)", 1_000_000, || {
        signal.with(|v| counter.fetch_add(*v as usize, Ordering::Relaxed));
    });

    let signal = cx.create_signal("same".to_string());
    bench("signal.set_eq (equal: skipped)", 1_000_000, || {
        let _ = signal.set_eq("same".to_string());
    });

    // --- Derived -----------------------------------------------------------
    let source = cx.create_signal(41u32);
    let doubled = derived(&source, |v| v * 2);
    bench("derived.get (x2)", 1_000_000, || {
        let _ = doubled.get();
    });

    let source = cx.create_signal(vec![1u64; 16]);
    let len = derived(&source, |v: &Vec<u64>| v.len());
    bench("derived.get (vec len)", 1_000_000, || {
        let _ = len.get();
    });

    // --- Cancellation ------------------------------------------------------
    let token = CancellationToken::new();
    bench("token.child_token + drop", 100_000, || {
        let _ = token.child_token();
    });

    let token = CancellationToken::new();
    bench("token.cancel (no waiters)", 1_000_000, || {
        let clone = token.clone();
        clone.cancel();
    });

    bench("token.guard (cancel on drop)", 1_000_000, || {
        let _ = token.guard();
    });

    // --- Local task queue --------------------------------------------------
    bench("spawn_local_task + run", 10_000, || {
        let task = quoin_core::spawn_local_task(async { 1u32 });
        let _ran = quoin_core::local::run_local_tasks();
        let _ = quoin_core::pool::block_on(task);
    });

    // --- Pooled executor ---------------------------------------------------
    let rc = Rc::new(());
    let rc2 = rc.clone();
    drop(rc2);
    drop(rc);
    bench("pool.spawn_pooled (trivial fut)", 10_000, || {
        quoin_core::pool::spawn_pooled(async { 1u32 });
    });
    // Let the pool drain before exiting.
    std::thread::sleep(std::time::Duration::from_millis(200));

    println!("\nbaseline recorded — compare across commits on the same machine.");
}
