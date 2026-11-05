//! `component!` and `run_app!` for the Floem backend.
//!
//! Floem has no component runtime — the view tree is built once and updated
//! through signals — so `component!` expands to a plain function that
//! creates its signals once and returns the view. `run_app!` expands to a
//! `main` that launches the Floem application with the component's view.
//!
//! When the component declares `keys { .. }`, the returned root view is made
//! focusable, grabs focus on construction (the Floem counterpart of the
//! GPUI backend's `track_focus` contract: shortcut-like keys also reach the
//! listener registry without focus, typing keys need it), and carries one
//! `KeyDown` listener dispatching each parsed keystroke to its handler.

use crate::parse::ComponentAst;
use crate::run_app::RunAppInput;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST into a Floem view function.
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

    // `keys { .. }`: focus the root and dispatch each parsed keystroke.
    // The `move` closure owns its own clones of the state signals the
    // handlers reference, so the component's own bindings stay live.
    let (key_shadows, keys_decorator) = if ast.keys.is_empty() {
        (Vec::new(), quote! {})
    } else {
        let state_globals: std::collections::HashSet<String> = ast
            .state
            .iter()
            .map(|s| s.name.to_string())
            .chain(ast.globals.iter().map(|g| g.name.to_string()))
            .collect();

        let mut shadows = Vec::new();
        let mut arms = Vec::new();
        for key in &ast.keys {
            let arm = super::keys::lower_key_entry(&key.keystroke, &key.handler, &state_globals);
            shadows.extend(arm.shadows);
            arms.push(arm.arm);
        }

        let decorator = quote! {
            {
                use ::floem::views::Decorators as _;
                root_view
                    .style(|__s| __s.focusable())
                    .request_focus(|| ())
                    .on_event(
                        ::floem::event::listener::KeyDown,
                        move |__cx, __ev| {
                            let _ = __cx;
                            #(#arms)else*
                            else {
                                ::floem::event::EventPropagation::Continue
                            }
                        },
                    )
            }
        };
        (shadows, decorator)
    };

    let body = if ast.keys.is_empty() {
        quote! { { #(#render_stmts)* } }
    } else {
        quote! {
            {
                #(#key_shadows)*
                {
                    let root_view = { #(#render_stmts)* };
                    #keys_decorator
                }
            }
        }
    };

    quote! {
        #[allow(non_snake_case)]
        #vis fn #name() -> impl ::floem::view::IntoView {
            use quoin::ReactiveContext;
            use quoin::Signal;
            let ctx = quoin::FloemContext::new();
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            #(#on_mount_tokens)*
            #(#state_clones)*
            #(#global_clones)*
            #body
        }
    }
}

#[must_use]
/// Expands `run_app!` into the Floem application bootstrap (`main`).
pub fn emit_run_app(input: &RunAppInput) -> TokenStream {
    let component = &input.component;
    quote! {
        fn main() {
            ::floem::launch(#component);
        }
    }
}
