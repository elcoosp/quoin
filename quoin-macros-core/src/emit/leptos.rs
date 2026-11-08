use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST into a Leptos component function with signal
/// initialization, action closures, lifecycle hooks, and a reactive render
/// closure.
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = ctx.create_signal(#default);
        }
    });

    let action_closures = crate::emit::common::action_closure_tokens(ast);

    let on_mount_tokens: Vec<TokenStream> = match &ast.on_mount {
        Some(block) => block.stmts.iter().map(|s| quote! { #s }).collect(),
        None => Vec::new(),
    };

    let on_unmount_tokens: TokenStream = match &ast.on_unmount {
        Some(block) => {
            let stmts: Vec<TokenStream> = block.stmts.iter().map(|s| quote! { #s }).collect();
            quote! {
                leptos::prelude::on_cleanup(move || {
                    #(#stmts)*
                });
            }
        }
        None => quote! {},
    };

    // `keys { .. }` has no web routing yet — the documented diagnostic
    // contract (docs/BACKEND-PARITY.md), never a silent drop.
    let keys_error = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! {
            ::core::compile_error!(
                "keys { .. } is not supported on the Leptos backend (no global key \
                 routing in v1); attach key events on the specific widget instead \
                 (see docs/BACKEND-PARITY.md)"
            );
        }
    };

    let global_inits: Vec<_> = ast
        .globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            let fty = &g.ty;
            quote! {
                let #fname: Option<quoin::LeptosSignal<#fty>> = ctx.use_global::<#fty>();
            }
        })
        .collect::<Vec<_>>();

    // FIXED: Shadow-clone all state signals and globals BEFORE the outer
    // move || closure. Any inner move || inside render_stmts (e.g.
    // `prop:value={move || filter_text.get()}`) then moves its own clone,
    // leaving the outer closure's copy intact — making the outer closure
    // FnMut instead of FnOnce.
    let state_clones = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { let #fname = #fname.clone(); }
    });
    let global_clones = ast.globals.iter().map(|g| {
        let fname = &g.name;
        quote! { let #fname = #fname.clone(); }
    });

    let render_stmts = &ast.render.stmts;

    quote! {
        #[leptos::prelude::component]
        #vis fn #name() -> impl leptos::prelude::IntoView {
            use quoin::ReactiveContext;
            use quoin::Signal;
            use leptos::prelude::ElementChild;
            #keys_error
            let ctx = quoin::LeptosContext::new();
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            #(#on_mount_tokens)*
            #on_unmount_tokens
            #(#state_clones)*
            #(#global_clones)*
            move || { #(#render_stmts)* }
        }
    }
}
// ---------------------------------------------------------------------------
