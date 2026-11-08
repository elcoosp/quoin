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
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    println!("{name:<46} {iterations:>9} ops in {elapsed:>10.2?}  ({ns_per_op:>9.0} ns/op)");
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
        });
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
    // The changed path is where the single-lock `set_if_eq` override pays
    // off: one write-lock acquisition instead of a read lock plus a write
    // lock (see docs/BENCHMARKS.md).
    let mut flip = 0u32;
    bench("signal.set_eq (changed: written)", 1_000_000, || {
        flip += 1;
        let _ = signal.set_eq(if flip.is_multiple_of(2) {
            "same".to_string()
        } else {
            "other".to_string()
        });
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

    // --- Memoized derived (cached) -----------------------------------------
    let source = cx.create_signal(vec![7u64; 64]);
    let sum = memo(&source, |values: &Vec<u64>| values.iter().sum::<u64>());
    bench("memo.get (cached, 64-elem sum)", 1_000_000, || {
        let _ = sum.get();
    });

    source.set(vec![8u64; 64]);
    bench("memo.get (uncached after change)", 1_000_000, || {
        source.set(vec![9u64; 64]);
        let _ = sum.get();
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
    bench("pool.spawn_pooled (trivial fut)", 10_000, || {
        quoin_core::pool::spawn_pooled(async { 1u32 });
    });
    // Let the pool drain before exiting.
    std::thread::sleep(std::time::Duration::from_millis(200));

    // --- Effects (dependency-tracking reactive runtime) ---------------------
    bench_effects(cx);

    // --- Effect disposal at scale -------------------------------------------
    bench_disposal();

    // --- Type-erased signals (AnySignal dispatch overhead) ------------------
    bench_erasure(cx, &counter);

    println!("\nbaseline recorded — compare across commits on the same machine.");
}

/// Effect runtime gates and cycle costs (M1.6): the zero-cost gates are the
/// regression baseline — tracked reads/writes must match the plain numbers
/// above when no effect is wired.
fn bench_effects(cx: quoin_core::test_support::StaticContext) {
    println!();
    // Reads outside any effect run: the tracking gate is one relaxed atomic
    // load, so these must stay close to the plain-signal numbers above.
    let signal = cx.create_signal(1u32);
    bench("signal.get (tracked cell, no effects)", 1_000_000, || {
        let _ = signal.get();
    });

    let signal = cx.create_signal(0u32);
    bench("signal.set (tracked cell, no effects)", 1_000_000, || {
        signal.set(1);
    });

    // Live effects exist but this cell is unobserved: the notify path pays
    // the subscriber-list lock and finds it empty.
    let unobserved = cx.create_signal(0u32);
    let _idle_effect = quoin_core::create_effect(|| {});
    bench(
        "signal.set (effect exists, cell unobserved)",
        1_000_000,
        || {
            unobserved.set(1);
        },
    );

    // Observed cell: each set schedules and runs a trivial effect body.
    let observed = cx.create_signal(0u32);
    let observed2 = observed.clone();
    let _watching = quoin_core::create_effect(move || {
        let _value = observed2.get();
    });
    bench("signal.set (trivial effect re-runs)", 1_000_000, || {
        observed.set(1);
    });

    // Effect creation cost including the initial run.
    bench("effect.create (initial run, 1 dep)", 100_000, || {
        let dep = cx.create_signal(0u32);
        let dep2 = dep.clone();
        let handle = quoin_core::create_effect(move || {
            let _value = dep2.get();
        });
        handle.dispose();
    });

    // Batched writes coalesce N mutations into one re-run.
    let observed = cx.create_signal(0u32);
    let observed2 = observed.clone();
    let _watching = quoin_core::create_effect(move || {
        let _value = observed2.get();
    });
    bench("effect rerun (batched set x10 → 1 run)", 100_000, || {
        cx.batch(|| {
            for _ in 0..10 {
                observed.set(1);
            }
        });
    });

    // watch: subscription + one callback per write.
    let watched = cx.create_signal(0u32);
    let seen = Arc::new(AtomicUsize::new(0));
    let (watched2, seen2) = (watched.clone(), seen.clone());
    let _handle = quoin_core::watch(&watched2, move |value: &u32| {
        seen2.store(*value as usize, Ordering::Relaxed);
    });
    bench("watch callback (set → callback)", 1_000_000, || {
        watched.set(1);
    });
}

/// Effect disposal at scale: removing a node from the runtime registry must
/// not get more expensive as the number of live detached effects grows —
/// `dispose` is O(1) (slot-indexed swap-remove), so both rows should land in
/// the same ballpark.
fn bench_disposal() {
    println!();
    for live in [100u32, 1_000] {
        let mut handles: Vec<quoin_core::EffectHandle> = (0..live)
            .map(|_| quoin_core::create_effect(|| {}))
            .collect();
        bench(
            &format!("effect.dispose ({live} live effects)"),
            live,
            move || {
                if let Some(handle) = handles.pop() {
                    handle.dispose();
                }
            },
        );
    }
}

/// `AnySignal` dispatch overhead: compare each row with the plain-signal
/// equivalent above — reads should be indistinguishable.
fn bench_erasure(cx: quoin_core::test_support::StaticContext, counter: &Arc<AtomicUsize>) {
    println!();
    let signal = cx.create_signal(1u32);
    let erased = quoin_core::AnySignal::new(signal.clone());
    bench("any_signal.get (vtable dispatch)", 1_000_000, || {
        let _ = erased.get();
    });
    let signal = cx.create_signal(0u32);
    let erased = quoin_core::AnySignal::new(signal.clone());
    bench("any_signal.set (vtable dispatch)", 1_000_000, || {
        erased.set(1);
    });
    let signal = cx.create_signal(1u32);
    let erased = quoin_core::AnySignal::new(signal.clone());
    bench("any_signal.with (clone + lend)", 1_000_000, || {
        erased.with(|v| counter.fetch_add(*v as usize, Ordering::Relaxed));
    });
}
