//! Compile-time verification of the `gpui-rsx-classes` engine expansions.
//!
//! Every `quoin_render!` here must *type-check* against the vendored GPUI
//! stub (whose builder surface is generated from quoin's pinned
//! `gpui-pre@0.3.10` snapshot). This proves the emitted chains reference
//! methods that actually exist, without paying the multi-minute gpui-pre
//! build. Rendering behavior is covered by the real-GPUI crates.
//!
//! Coverage:
//! - quoin-known classes (byte-parity path): layout, spacing, palette colors,
//!   theme tokens, hover, scroll overflow;
//! - newly gained vocabulary: extended palette, fractions, arbitrary values,
//!   negative utilities, `border-t-2`;
//! - newly gained variants (`focus:`, `active:`, `group-hover:`) including
//!   the automatic stateful `.id(..)` injection;
//! - dynamic `class:` expressions through the inline `rsx_runtime` module;
//! - `h1` defaults, `on_click` handlers, `if[..]`/`for[..]` control flow.

use gpui::prelude::*;
use quoin_macros::quoin_render;

fn sample_items() -> Vec<String> {
    vec!["alpha".to_owned(), "beta".to_owned()]
}

fn main() {
    // ── quoin-known classes: existing engine, byte-identical output ────────
    let known = quoin_render! {
        div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white hover:bg-gray-800") {
            h1() { "Byte parity" }
            div(class: "text-2xl font-bold rounded-md border-b w-full h-full opacity-50 truncate") { "ok" }
        }
    };

    // Scroll overflow keeps the stateful `.id()` injection.
    let scroll = quoin_render! {
        div(class: "flex flex-col overflow-y-scroll gap-2") {
            "scrolled"
        }
    };

    // ── Theme tokens: baked shadcn-dark hexes, unchanged ────────────────────
    let tokens = quoin_render! {
        div(class: "bg-primary text-muted-foreground bg-secondary hover:bg-secondary") {
            "tokens"
        }
    };

    // ── New vocabulary: classes that were compile errors before ────────────
    let extended = quoin_render! {
        div(class: "text-blue-400 w-1/2 border-t-2 gap-[14px] bg-[#0b0b0b]") {
            "extended vocabulary"
        }
    };

    // ── New variants: focus / active / group-hover (auto stateful) ─────────
    let variants = quoin_render! {
        div(class: "focus:border-blue-500 active:bg-blue-700 group-hover:bg-red-500 hover:text-blue-400") {
            "variants"
        }
    };

    // ── Dynamic classes: inline rsx_runtime module + render-time matcher ───
    let accent = String::from("flex text-blue-400");
    let dynamic = quoin_render! {
        div(class: accent.clone()) {
            "dynamic"
        }
    };

    // Dynamic class inside control flow: the runtime module is hoisted to
    // the render root once, the refinement applies per element.
    let items = sample_items();
    let dynamic_control_flow = quoin_render! {
        div(class: "flex flex-col") {
            if[items.len() > 1] {
                div(class: "w-1/2") { "branch" }
            } else {
                div(class: "w-full") { "empty" }
            }
            for[item in items.clone()] {
                div(class: accent.clone()) { item }
            }
        }
    };

    // ── Handlers keep working (wrap_handler closure machinery) ─────────────
    let counter = 0u32;
    let handler = quoin_render! {
        div(class: "flex cursor-pointer", on_click: move |_| { let _ = counter; }) {
            "click me"
        }
    };

    // Mixed string: quoin-known + engine-resolved classes in one argument.
    let mixed = quoin_render! {
        div(class: "flex flex-col gap-4 text-blue-400 hover:text-blue-300") {
            "mixed engines, one class string"
        }
    };

    let _ = (known, scroll, tokens, extended, variants, dynamic, dynamic_control_flow, handler, mixed);
}
