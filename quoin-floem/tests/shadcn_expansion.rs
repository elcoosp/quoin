//! Canonical shadcn surface — real expansion compile tests on the Floem
//! backend (rich-elements Phase 2).
//!
//! This file is compiled twice by the suite runner:
//!
//! - `cargo nextest run -p quoin-floem` — against the **generic** lowerings
//!   (plain Floem views over quoin theme tokens);
//! - `cargo nextest run -p quoin-floem --features shadcn` — against the
//!   **shadcn** lowerings (real `floem-shadcn` components, sibling checkout).
//!
//! Both cfg branches must accept the same source: the Tier-1 surface
//! (`card` family, `alert`, `spinner`, `kbd`, `avatar`) plus the Phase 2
//! Tier-2 additions (`accordion` family, `tabs`, `tooltip`). Compile-only by
//! design — the view function is never launched, but every expansion has to
//! typecheck to build the test binary.

use floem::prelude::*;
use quoin::quoin_render;
use quoin::Signal as _;
use quoin::test_support::static_signal;

/// A controlled `expanded:` state for the accordion family.
fn test_expanded() -> quoin::test_support::StaticSignal<Option<String>> {
    static_signal(Some("item-1".to_string()))
}

/// A controlled `active:` state for `tabs`.
fn test_active() -> quoin::test_support::StaticSignal<usize> {
    static_signal(1usize)
}

/// The full Tier-1 + Tier-2 surface in one tree.
pub fn shadcn_surface_view() -> impl IntoView {
    let expanded = test_expanded();
    let active = test_active();

    quoin_render! {
        div(class: "flex flex-col gap-4") {
            card() {
                card_header(
                    title: "Phase 2 surface",
                    description: "One source, two lowerings",
                )
                card_action() {
                    button(variant: "outline", size: "sm", label: "Edit")
                }
                card_content() {
                    alert(title: "Heads up", description: "Compose freely.")
                    alert(variant: "destructive", title: "Deleted")
                    alert_title() { "Standalone title" }
                    alert_description() { "Standalone description" }
                    spinner()
                    kbd(text: "Ctrl+K")
                    avatar(fallback: "AL", size: 40.0)
                    avatar_fallback() { "AL" }
                }
                card_footer() {
                    badge(variant: "secondary") { "shimmed" }
                }
            }
            accordion(expanded: expanded.get()) {
                accordion_item(value: "item-1") {
                    accordion_trigger() { "Is it tested?" }
                    accordion_content() { "Both cfg branches compile this." }
                }
                accordion_item(value: "item-2") {
                    accordion_trigger() { "Is it themed?" }
                    accordion_content() { "Shadcn tokens on both paths." }
                }
            }
            tabs(active: active.get(), index: 0, on_click: move |i| active.set(i)) {
                tab(label: "Account", index: 0, on_click: move |i| active.set(i)) {
                    text() { "Account body" }
                }
                tab(label: "Password", index: 1) {
                    text() { "Password body" }
                }
            }
            tooltip(text: "Hovering works on both paths") {
                button(label: "Hover me")
            }
        }
    }
}

/// An uncontrolled accordion (no `expanded:` binding) must compile too —
/// the lowering owns its state internally.
pub fn uncontrolled_accordion_view() -> impl IntoView {
    quoin_render! {
        accordion() {
            accordion_item(value: "solo") {
                accordion_trigger() { "Self-managed" }
                accordion_content() { "No binding required." }
            }
        }
    }
}

#[test]
fn shadcn_surface_expansion_typechecks() {
    // Building the views exercises the full generated code; the harness
    // never launches a Floem app.
    let _ = shadcn_surface_view;
    let _ = uncontrolled_accordion_view;
}
