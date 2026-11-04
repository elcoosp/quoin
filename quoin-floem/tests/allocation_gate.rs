//! Allocation gate for the Floem backend (gate 2 of the zero-overhead
//! contract).
//!
//! The counting allocator (dhat) measures how many heap allocations the
//! construction of a fully static `quoin_render!` tree performs, and asserts
//! it costs exactly what the hand-written equivalent — the literal token
//! stream the macro emits — costs. The plan's contract: the expansion
//! contains only native framework calls, so it must not add a single
//! allocation over what an application would write by hand.
//!
//! # Why the minimum over repeated runs
//!
//! Floem's reactive runtime defers work: constructing a view schedules
//! effects whose allocations land inside whatever measurement window runs
//! next, so a single-shot delta is noisy (the same token sequence measured
//! twice differs by a few blocks). The *minimum* over repeated constructions
//! is the pure construction cost with the deferred-work noise filtered out,
//! and it is byte-stable across process runs — which the equality gate
//! needs.
//!
//! Run with: `cargo test -p quoin-floem --test allocation_gate`

// dhat installs the counting global allocator for this test binary.
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use floem::prelude::*;
use quoin_macros::quoin_render;

/// The macro-expanded static tree: classes fully lowered at compile time,
/// native `Stack`/`Label` constructors, no dynamic seams.
fn macro_view() -> impl floem::view::IntoView {
    quoin_render! {
        div(class: "flex flex-col gap-2 p-4") {
            h1() { "Counter" }
            p() { "Static text" }
        }
    }
}

/// The hand-written twin — exactly the tokens the expansion above produces
/// (see `quoin-macros-core/tests/floem_expansion.rs` for the shape checks).
fn hand_view() -> impl floem::view::IntoView {
    ::floem::views::Stack::vertical((
        ::floem::views::Label::derived(|| "Counter").style(|__s| __s.font_size(24.0)),
        ::floem::views::Label::derived(|| "Static text"),
    ))
    .style(|__s| __s.gap(8.0).padding(16.0))
}

/// The one dhat profiler this test binary may run (dhat supports a single
/// live profiler; tests share it through a `OnceLock`).
fn profiler() -> &'static dhat::Profiler {
    static PROFILER: std::sync::OnceLock<dhat::Profiler> = std::sync::OnceLock::new();
    PROFILER.get_or_init(|| dhat::Profiler::builder().testing().build())
}

/// Measures the heap allocations one call to `f` performs.
fn allocations<T>(f: impl FnOnce() -> T) -> u64 {
    profiler();
    let before = dhat::HeapStats::get().total_blocks;
    let _value = f();
    let after = dhat::HeapStats::get().total_blocks;
    after - before
}

/// The minimum allocation cost of `f` over `n` constructions.
fn min_cost(f: fn() -> u64, n: usize) -> u64 {
    (0..n).map(|_| f()).min().unwrap_or_default()
}

#[test]
fn static_tree_costs_exactly_the_hand_written_view() {
    fn hand() -> u64 {
        allocations(hand_view)
    }
    fn macro_path() -> u64 {
        allocations(macro_view)
    }

    // Warm-up outside the measured region: lazy statics, std init, dhat
    // bookkeeping — anything that runs once must not skew the minima.
    let _ = allocations(|| ());
    for _ in 0..3 {
        let _ = (hand(), macro_path());
    }

    let hand_cost = min_cost(hand, 15);
    let macro_cost = min_cost(macro_path, 15);

    assert_eq!(
        macro_cost, hand_cost,
        "the quoin_render! expansion must not allocate more than the \
         hand-written equivalent (macro: {macro_cost}, hand: {hand_cost})"
    );
}
