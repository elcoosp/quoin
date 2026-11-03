//! `component!` and `run_app!` for the Floem backend.
//!
//! Floem has no component runtime — the view tree is built once and updated
//! through signals — so `component!` expands to a plain function that
//! creates its signals once and returns the view. `run_app!` expands to a
//! `main` that launches the Floem application with the component's view.

use crate::parse::ComponentAst;
use crate::run_app::RunAppInput;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST into a Floem view function.
#[cfg_attr(
    all(
        feature = "floem",
        any(
            feature = "gpui",
            feature = "leptos",
            feature = "dioxus",
            feature = "xilem"
        )
    ),
    expect(dead_code)
)]
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! { let #fname = ctx.create_signal(#default); }
    });

    let action_closures = crate::emit::common::action_closure_tokens(ast);

    let on_mount_tokens: Vec<TokenStream> = match &ast.on_mount {
        Some(block) => block.stmts.iter().map(|s| quote! { #s }).collect(),
        None => Vec::new(),
    };

    let global_inits: Vec<_> = ast
        .globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            let fty = &g.ty;
            quote! { let #fname: Option<quoin::FloemSignal<#fty>> = ctx.use_global::<#fty>(); }
        })
        .collect();

    // Shadow-clone the state signals and globals before the returned view
    // expression moves them into its closures (same shape as the Leptos
    // backend).
    let state_clones = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { let #fname = #fname.clone(); }
    });
    let global_clones = ast.globals.iter().map(|g| {
        let fname = &g.name;
        quote! { let #fname = #fname.clone(); }
    });

    let render_stmts = &ast.render.stmts;

    let keys_error = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! {
            ::core::compile_error!("`keys { .. }` on the Floem backend lands with the M5 keys push; handle input on the specific widget for now");
        }
    };

    quote! {
        #vis fn #name() -> impl ::floem::view::IntoView {
            #keys_error
            use quoin::ReactiveContext;
            use quoin::Signal;
            let ctx = quoin::FloemContext::new();
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            #(#on_mount_tokens)*
            #(#state_clones)*
            #(#global_clones)*
            { #(#render_stmts)* }
        }
    }
}

#[must_use]
/// Expands `run_app!` into the Floem application bootstrap (`main`).
#[cfg_attr(
    all(
        feature = "floem",
        any(
            feature = "gpui",
            feature = "leptos",
            feature = "dioxus",
            feature = "xilem"
        )
    ),
    expect(dead_code)
)]
pub fn emit_run_app(input: &RunAppInput) -> TokenStream {
    let component = &input.component;
    quote! {
        fn main() {
            ::floem::launch(#component);
        }
    }
}
