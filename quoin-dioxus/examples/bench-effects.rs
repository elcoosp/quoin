//! Micro-benchmarks for the Dioxus effect path — the core effect graph
//! observed through the bridged [`quoin_dioxus::DioxusSignal`].
//!
//! Same method as `quoin-xilem/examples/bench-effects.rs`: no benchmark
//! framework, plain `std::time::Instant` loops in release mode.
//!
//! ```bash
//! just bench-dioxus      # cargo run --release -p quoin-dioxus --example bench-effects
//! ```
//!
//! What is measured (the bridged-signal surfaces, in order):
//!
//! - the signal baseline with no effects wired — the zero-cost gate (one
//!   thread-local flag check per read on top of the dioxus read);
//! - effect creation over the bridge (the dep-cell subscription path);
//! - the observed-write cycle: a write that notifies dioxus subscribers and
//!   schedules a re-run of the core effect that read the signal;
//! - batch coalescing over that cycle (10 writes, one re-run).
//!
//! Numbers are machine-relative regression baselines, not absolute truth.

use dioxus_core::{Element, ScopeId, VNode, VirtualDom};
use quoin_core::{ReactiveContext, ReactiveContextExt, SignalExt};
use quoin_dioxus::{DioxusContext, DioxusSignal};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

fn app() -> Element {
    VNode::empty()
}

thread_local! {
    static BENCH_VDOM: std::cell::RefCell<Option<&'static VirtualDom>> =
        const { std::cell::RefCell::new(None) };
}

fn with_vdom<R>(f: impl FnOnce() -> R) -> R {
    BENCH_VDOM.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            let mut vdom = VirtualDom::new(app);
            vdom.rebuild_in_place();
            *opt = Some(Box::leak(Box::new(vdom)));
        }
        let vdom = opt.expect("VirtualDom leaked above");
        vdom.in_scope(ScopeId::ROOT, f)
    })
}

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
    println!("quoin-dioxus effect-path micro-benchmarks (release build)\n");

    let cx = DioxusContext::new();
    let sink = Arc::new(AtomicUsize::new(0));

    // --- Signal baseline (no effects wired) --------------------------------
    let signal = with_vdom(|| cx.create_signal(0u32));
    bench("dioxus_signal.set (no effects)", 1_000_000, || {
        signal.set(1);
    });

    let signal = with_vdom(|| cx.create_signal(1u32));
    bench("dioxus_signal.get (clone u32)", 1_000_000, || {
        let _ = signal.get();
    });

    let signal = with_vdom(|| cx.create_signal(0u32));
    bench("dioxus_signal.set_eq (equal: skipped)", 1_000_000, || {
        let _ = signal.set_eq(0u32);
    });

    // --- Effect creation over the bridge ------------------------------------
    let signal: DioxusSignal<u32> = with_vdom(|| cx.create_signal(0u32));
    let sink2 = Arc::clone(&sink);
    bench("effect.create (bridged, 1 dep)", 100_000, || {
        let (signal, sink) = (signal.clone(), Arc::clone(&sink2));
        let _handle = quoin_core::create_effect(move || {
            let _value = signal.get();
            sink.store(0, Ordering::Relaxed);
        });
    });

    // --- The observed-write cycle (the bridge hot path) ---------------------
    let signal = with_vdom(|| cx.create_signal(0u32));
    let sink2 = Arc::clone(&sink);
    let (signal2, sink3) = (signal.clone(), Arc::clone(&sink2));
    let _handle = quoin_core::create_effect(move || {
        let value = signal2.get();
        sink3.store(value as usize, Ordering::Relaxed);
    });
    bench("dioxus_signal.set (effect re-runs)", 1_000_000, || {
        signal.set(1);
    });

    // --- Batch coalescing over the cycle ------------------------------------
    let signal = with_vdom(|| cx.create_signal(0u32));
    let sink2 = Arc::clone(&sink);
    let (signal2, sink3) = (signal.clone(), Arc::clone(&sink2));
    let _handle = quoin_core::create_effect(move || {
        let value = signal2.get();
        sink3.store(value as usize, Ordering::Relaxed);
    });
    bench("dioxus_signal.set (batched x10, 1 re-run)", 100_000, || {
        cx.batch(|| {
            for i in 0..10u32 {
                signal.set(i);
            }
        });
    });
}
