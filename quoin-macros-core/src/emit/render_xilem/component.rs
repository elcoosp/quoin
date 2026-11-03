//! `component!` and `run_app!` for the Xilem backend.
//!
//! Xilem re-runs the app logic on every action, so signals cannot live in a
//! render function. The v1 lowering therefore keeps the hand-written shape
//! the examples use — a state struct plus `Xilem::new_simple` with
//! `quoin_render!` inside the app logic — and `run_app!`/`component!` emit
//! precise diagnostics pointing there. The rebuild pump (see
//! `quoin-xilem/src/pump.rs`) installs through
//! [`quoin::xilem_pump::pump_view`], which the app logic includes in its
//! tree.

use crate::parse::ComponentAst;
use crate::run_app::RunAppInput;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST for Xilem: a precise diagnostic for now.
#[cfg_attr(
    all(
        feature = "xilem",
        any(
            feature = "gpui",
            feature = "leptos",
            feature = "dioxus",
            feature = "floem"
        )
    ),
    expect(dead_code)
)]
pub fn emit_component(_ast: &ComponentAst) -> TokenStream {
    quote! {
        ::core::compile_error!(
            "component! on the Xilem backend lands with the v1 push after the pump milestone; \
             for now, define a state struct owning your quoin signals and call quoin_render! \
             inside the app logic (see examples/counter-xilem)"
        )
    }
}

#[must_use]
/// Expands `run_app!` for Xilem: a precise diagnostic for now.
#[cfg_attr(
    all(
        feature = "xilem",
        any(
            feature = "gpui",
            feature = "leptos",
            feature = "dioxus",
            feature = "floem"
        )
    ),
    expect(dead_code)
)]
pub fn emit_run_app(_input: &RunAppInput) -> TokenStream {
    quote! {
        ::core::compile_error!(
            "run_app! on the Xilem backend lands with the v1 push after the pump milestone; \
             for now, boot with `Xilem::new_simple(state, |state| { ... quoin_render! { ... } }, \
             WindowOptions::default())` and include `quoin::xilem_pump::pump_view()` in the tree \
             (see examples/counter-xilem)"
        )
    }
}
