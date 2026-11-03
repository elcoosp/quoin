//! Allocation gate (gate 2 of the zero-overhead contract).
//!
//! The counting allocator (dhat) measures how many heap allocations the
//! construction of a fully static `quoin_render!` tree performs, and asserts
//! it costs exactly what the hand-written equivalent — the literal token
//! stream the macro emits — costs. The plan's contract: the expansion contains
//! only native framework calls, so it must not add a single allocation over
//! what an application would write by hand.
//!
//! Run with: `cargo test -p quoin-xilem --test allocation_gate`

// dhat installs the counting global allocator for this test binary.
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use quoin_macros::quoin_render;

/// Marker app state for the view generics.
struct AppState;

/// The macro-expanded static tree: classes fully lowered at compile time,
/// native `flex`/`label`/`sized_box` constructors, no dynamic seams.
fn macro_view() -> impl xilem::WidgetView<AppState> {
    quoin_render! {
        div(class: "flex flex-col gap-2 p-4") {
            h1() { "Counter" }
            p() { "Static text" }
        }
    }
}

/// The hand-written twin — exactly the tokens the expansion above produces
/// (see `quoin-macros-core/tests/xilem_expansion.rs` for the shape checks).
fn hand_view() -> impl xilem::WidgetView<AppState> {
    use xilem::style::Style as _;
    xilem::view::sized_box(
        xilem::view::flex(
            xilem::kurbo::Axis::Vertical,
            (
                xilem::view::label("Counter").text_size(24f32),
                xilem::view::label("Static text"),
            ),
        )
        .gap(xilem::masonry::layout::Length::px(8f64)),
    )
    .padding(xilem::masonry::properties::Padding {
        top: xilem::masonry::layout::Length::px(16f64),
        bottom: xilem::masonry::layout::Length::px(16f64),
        left: xilem::masonry::layout::Length::px(16f64),
        right: xilem::masonry::layout::Length::px(16f64),
    })
}

/// The one dhat profiler this test binary may run (dhat supports a single
/// live profiler; tests share it through a `OnceLock`).
fn profiler() -> &'static dhat::Profiler {
    static PROFILER: std::sync::OnceLock<dhat::Profiler> = std::sync::OnceLock::new();
    PROFILER.get_or_init(|| dhat::Profiler::builder().testing().build())
}

/// Measures the heap allocations one call to `f` performs.
fn allocations<T>(f: impl FnOnce() -> T) -> (T, u64) {
    profiler();
    let before = dhat::HeapStats::get().total_blocks;
    let value = f();
    let after = dhat::HeapStats::get().total_blocks;
    (value, after - before)
}

#[test]
fn static_tree_costs_exactly_the_hand_written_view() {
    // Warm-up outside the measured region: lazy statics, std init, dhat
    // bookkeeping — anything that runs once must not skew the delta.
    let _ = allocations(hand_view);
    let _ = allocations(macro_view);

    let (_, macro_cost) = allocations(macro_view);
    let (_, hand_cost) = allocations(hand_view);

    assert_eq!(
        macro_cost, hand_cost,
        "the quoin_render! expansion must not allocate more than the \
         hand-written equivalent (macro: {macro_cost}, hand: {hand_cost})"
    );
}
