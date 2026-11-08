//! Micro-benchmarks for the Xilem effect path — the runtime the `effect!`
//! macro produces on this backend.
//!
//! Same method as `quoin-core/examples/bench.rs`: no benchmark framework,
//! plain `std::time::Instant` loops in release mode.
//!
//! ```bash
//! just bench-xilem       # cargo run --release -p quoin-xilem --example bench-effects
//! ```
//!
//! What is measured (the `effect!` lowering surfaces, in order):
//!
//! - the signal baseline (`XilemSignal` = `NotifierSignal`) with no effects
//!   wired — the zero-cost gate;
//! - effect creation through both routing paths (detached and inside a
//!   component effect scope — the thread-local lookup the `component!`
//!   expansion activates);
//! - the observed-write cycle: a signal write that schedules and flushes a
//!   re-run of the effect that read it (the `effect!` hot path);
//! - batch coalescing over that cycle (10 writes, one re-run);
//! - the armed-cleanup re-run shape the `cleanup:` lowering emits;
//! - `CleanupSlot::run` standalone.
//!
//! Numbers are machine-relative regression baselines, not absolute truth.

use quoin_core::{ReactiveContext, ReactiveContextExt, Signal, SignalExt};
use quoin_xilem::effect::{CleanupSlot, enter_effect_scope};
use quoin_xilem::{XilemContext, XilemSignal};
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
    println!("quoin-xilem effect-path micro-benchmarks (release build)\n");

    let runtime = Arc::new(xilem::tokio::runtime::Runtime::new().expect("tokio runtime"));
    let cx = XilemContext::new(runtime);
    let sink = Arc::new(AtomicUsize::new(0));

    // --- Signal baseline (no effects wired) --------------------------------
    let signal = cx.create_signal(0u32);
    bench("xilem_signal.set (no effects)", 1_000_000, || {
        signal.set(1);
    });

    let signal = cx.create_signal(1u32);
    bench("xilem_signal.get (clone u32)", 1_000_000, || {
        let _ = signal.get();
    });

    let signal = cx.create_signal(0u32);
    bench("xilem_signal.set_eq (equal: skipped)", 1_000_000, || {
        let _ = signal.set_eq(0u32);
    });

    // --- Effect creation (both routing paths) ------------------------------
    let signal = XilemSignal::new(0u32, || {});
    let sink2 = Arc::clone(&sink);
    bench("effect.create (detached, 1 dep)", 100_000, || {
        let (signal, sink) = (signal.clone(), Arc::clone(&sink2));
        let _handle = quoin_xilem::effect::create_effect(move || {
            let _value = signal.get();
            sink.store(0, Ordering::Relaxed);
        });
    });

    let signal = XilemSignal::new(0u32, || {});
    let scope = Arc::new(quoin_core::EffectScope::new());
    let sink2 = Arc::clone(&sink);
    bench("effect.create (component scope, 1 dep)", 100_000, || {
        let (signal, sink) = (signal.clone(), Arc::clone(&sink2));
        let guard = enter_effect_scope(Arc::clone(&scope));
        let _handle = quoin_xilem::effect::create_effect(move || {
            let _value = signal.get();
            sink.store(0, Ordering::Relaxed);
        });
        guard.pop();
    });

    // --- The observed-write cycle (the effect! hot path) -------------------
    let signal = cx.create_signal(0u32);
    let sink2 = Arc::clone(&sink);
    let (signal2, sink3) = (signal.clone(), Arc::clone(&sink2));
    let _handle = quoin_xilem::effect::create_effect(move || {
        let value = signal2.get();
        sink3.store(value as usize, Ordering::Relaxed);
    });
    bench("xilem_signal.set (effect re-runs)", 1_000_000, || {
        signal.set(1);
    });

    // --- Batch coalescing over the cycle ------------------------------------
    let signal = cx.create_signal(0u32);
    let sink2 = Arc::clone(&sink);
    let (signal2, sink3) = (signal.clone(), Arc::clone(&sink2));
    let _handle = quoin_xilem::effect::create_effect(move || {
        let value = signal2.get();
        sink3.store(value as usize, Ordering::Relaxed);
    });
    bench("xilem_signal.set (batched x10, 1 re-run)", 100_000, || {
        cx.batch(|| {
            for i in 0..10u32 {
                signal.set(i);
            }
        });
    });

    // --- The armed-cleanup re-run shape (cleanup: lowering) -----------------
    let signal = cx.create_signal(0u32);
    let slot = CleanupSlot::new();
    let sink2 = Arc::clone(&sink);
    slot.set(Box::new(move || {
        sink2.fetch_add(1, Ordering::Relaxed);
    }));
    let (signal2, slot2) = (signal.clone(), slot);
    let mut armed = false;
    let _handle = quoin_xilem::effect::create_effect(move || {
        if armed {
            slot2.run();
        }
        armed = true;
        let _ = signal2.get();
    });
    bench("set with cleanup: slot.run + body", 1_000_000, || {
        signal.set(1);
    });

    // --- CleanupSlot standalone ---------------------------------------------
    let slot = CleanupSlot::new();
    let sink2 = Arc::clone(&sink);
    slot.set(Box::new(move || {
        sink2.fetch_add(1, Ordering::Relaxed);
    }));
    bench("cleanup_slot.run", 1_000_000, || {
        slot.run();
    });
}
