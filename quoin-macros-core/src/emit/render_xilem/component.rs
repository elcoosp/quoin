//! `component!` and `run_app!` for the Xilem backend.
//!
//! Xilem re-runs the app logic on every action, so signals cannot live in a
//! render function. The lowering therefore gives the component exactly the
//! hand-written shape the framework wants (see `examples/counter-xilem`):
//!
//! - a struct holding one [`XilemSignal`] per `state` declaration (plus the
//!   `Option` globals), so the signals outlive the re-runs;
//! - `fn new(ctx: &XilemContext) -> Self`, which creates the signals and
//!   runs `on_mount`;
//! - `fn app_logic(&mut self) -> impl WidgetView<Self>`, which clones the
//!   signal handles, defines the action closures, and renders
//!   `fork(pump_view(), quoin_render! { … })` — the pump rides along in the
//!   tree so background signal writes request rebuilds (see
//!   `quoin::xilem_pump`).
//!
//! `run_app!` expands to a `main` that builds the context, installs the
//! pump notifier, constructs the component, and boots
//! `Xilem::new_simple_with_tokio` with the context's own runtime (one tokio
//! runtime serves the app's async work and the driver).
//!
//! The v1 action type is `()`: button callbacks mutate signals, they do not
//! dispatch framework actions. Non-`()` actions need the hand-written
//! `Xilem::new` path.

use crate::parse::ComponentAst;
use crate::run_app::RunAppInput;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;

#[must_use]
/// Expands a `component!` AST into the Xilem state struct + `new` + `app_logic`.
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;

    // `keys { .. }` has no Xilem routing yet — the documented diagnostic
    // contract (docs/BACKEND-PARITY.md), never a silent drop.
    let keys_error = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! {
            ::core::compile_error!(
                "keys { .. } is not supported on the Xilem backend (no global key routing \
                 in v1); handle input on the specific widget instead \
                 (see docs/BACKEND-PARITY.md)"
            );
        }
    };

    // `effect!` in the render body would re-create the effect on every
    // rebuild (app_logic re-runs per action) — a named diagnostic, never a
    // silent leak. Component-scope effects belong in `on_mount { .. }`.
    let effect_error = if render_body_invokes_effect(&ast.render) {
        quote! {
            ::core::compile_error!(
                "effect! inside the render body is not supported on the Xilem backend: \
                 app_logic re-runs on every rebuild, so the effect would be re-created \
                 each time; move it into the component's `on_mount { .. }` block, where \
                 it registers into the component's effect scope and is disposed with it \
                 (see docs/BACKEND-PARITY.md)"
            );
        }
    } else {
        quote! {}
    };

    let state_fields = ast.state.iter().map(|s| {
        let fname = &s.name;
        let fty = &s.ty;
        quote! { #vis #fname: ::quoin::XilemSignal<#fty> }
    });
    let global_fields = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! { #vis #fname: ::core::option::Option<::quoin::XilemSignal<#fty>> }
    });

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! { #fname: ctx.create_signal(#default) }
    });
    let global_inits = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! { #fname: ctx.use_global::<#fty>() }
    });

    let on_mount_tokens: Vec<TokenStream> = match &ast.on_mount {
        Some(block) => block.stmts.iter().map(|s| quote! { #s }).collect(),
        None => Vec::new(),
    };

    // Handles bound ahead of `on_mount` so the block sees the same names
    // the other backends' component scopes provide.
    let on_mount_locals = ast
        .state
        .iter()
        .map(|s| &s.name)
        .chain(ast.globals.iter().map(|g| &g.name))
        .map(|fname| {
            quote! {
                let #fname = ::core::clone::Clone::clone(&state.#fname);
            }
        });

    // app_logic clones every handle out of `self` — the framework re-runs
    // the logic per action, so the struct fields are the source of truth.
    let logic_clones = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { let #fname = ::core::clone::Clone::clone(&self.#fname); }
    });
    let global_logic_clones = ast.globals.iter().map(|g| {
        let fname = &g.name;
        quote! { let #fname = ::core::clone::Clone::clone(&self.#fname); }
    });

    let action_closures = crate::emit::common::action_closure_tokens(ast);
    let render_stmts = &ast.render.stmts;

    quote! {
        #keys_error

        #effect_error

        #vis struct #name {
            #(#state_fields,)*
            #(#global_fields,)*
            /// The component's effect scope: `effect!` invocations in
            /// `on_mount` register into it, and dropping the component
            /// disposes every effect (their final cleanup included).
            __quoin_effects: ::core::sync::Arc<::quoin_core::EffectScope>,
        }

        impl #name {
            /// Creates the component's signals from a Xilem context and
            /// runs the `on_mount` block once.
            #vis fn new(ctx: &::quoin::XilemContext) -> Self {
                use ::quoin::ReactiveContext as _;
                // The component effect scope: while `on_mount` runs below,
                // `effect!` invocations register into it (thread-local
                // current scope); the struct owns the scope so its `Drop`
                // disposes the effects with the component.
                let __quoin_effects =
                    ::core::sync::Arc::new(::quoin_core::EffectScope::new());
                let __quoin_scope_guard = ::quoin::xilem_effect::enter_effect_scope(
                    ::core::clone::Clone::clone(&__quoin_effects),
                );
                let state = Self {
                    #(#state_inits,)*
                    #(#global_inits,)*
                    __quoin_effects,
                };
                // The handles are in scope for `on_mount`, like on the
                // other backends. The block carries `allow(unused_variables)`:
                // every state field gets a handle whether or not this
                // component's `on_mount` reads it.
                #[allow(unused_variables)]
                {
                    #(#on_mount_locals)*
                    #(#on_mount_tokens)*
                }
                __quoin_scope_guard.pop();
                state
            }

            /// The Xilem app logic: clone the signal handles, define the
            /// action closures, and render inside the rebuild pump.
            ///
            /// The `use<Self>` precise capture keeps the returned opaque
            /// type free of the `&mut self` lifetime (edition 2024 RPITs
            /// capture all in-scope lifetimes), so the method satisfies
            /// `for<'a> FnMut(&mut Self) -> View` — the bound
            /// `Xilem::new_simple_with_tokio` puts on its app logic. The
            /// body only clones signal handles, so nothing borrows from
            /// `self` anyway.
            #vis fn app_logic(&mut self) -> impl ::xilem::WidgetView<#name> + use<> {
                use ::quoin::Signal as _;
                #(#logic_clones)*
                #(#global_logic_clones)*
                #(#action_closures)*
                // The pump rides along as a zero-size sibling of the app's
                // root view (xilem::core::fork would require the alongside
                // half to be element-free, which pump_view's canvas is not).
                ::xilem::view::flex(
                    ::xilem::kurbo::Axis::Vertical,
                    (
                        ::quoin::xilem_pump::pump_view::<#name>(),
                        { #(#render_stmts)* },
                    ),
                )
            }
        }
    }
}

/// Detects an `effect ! { .. }` invocation in the raw render block: the
/// token-level shape `effect` `!` brace-group, scanned recursively through
/// nested groups (an `effect != x` comparison does not match; a brace group
/// only follows in a macro invocation).
fn render_body_invokes_effect(block: &syn::Block) -> bool {
    fn scan(tokens: TokenStream) -> bool {
        #[derive(PartialEq)]
        enum State {
            None,
            SawEffect,
            SawBang,
        }
        let mut state = State::None;
        for tree in tokens {
            match tree {
                TokenTree::Group(group) => {
                    if state == State::SawBang && group.delimiter() == proc_macro2::Delimiter::Brace
                    {
                        return true;
                    }
                    if scan(group.stream()) {
                        return true;
                    }
                    state = State::None;
                }
                TokenTree::Ident(ref ident) if ident == "effect" => {
                    state = State::SawEffect;
                }
                TokenTree::Punct(ref punct) if punct.as_char() == '!' => {
                    if state == State::SawEffect {
                        state = State::SawBang;
                    } else {
                        state = State::None;
                    }
                }
                _ => state = State::None,
            }
        }
        false
    }
    let tokens: TokenStream = block.stmts.iter().map(|stmt| quote! { #stmt }).collect();
    scan(tokens)
}

#[must_use]
/// Expands `run_app!` into the Xilem application bootstrap (`main`).
pub fn emit_run_app(input: &RunAppInput) -> TokenStream {
    let component = &input.component;
    let window_opts = if let Some(expr) = &input.window_opts {
        quote! { #expr }
    } else {
        quote! { ::xilem::WindowOptions::new("quoin") }
    };
    quote! {
        fn main() -> ::core::result::Result<(), ::xilem::winit::error::EventLoopError> {
            let __quoin_runtime = ::std::sync::Arc::new(
                ::xilem::tokio::runtime::Runtime::new()
                    .expect("failed to start tokio runtime"),
            );
            let __quoin_ctx = ::quoin::XilemContext::new(
                ::core::clone::Clone::clone(&__quoin_runtime),
            );
            // Background signal writes request rebuilds through the pump.
            ::quoin::xilem_pump::install_notifier(&__quoin_ctx);
            let __quoin_state = #component::new(&__quoin_ctx);
            ::xilem::Xilem::new_simple_with_tokio(
                __quoin_state,
                move |__quoin_app: &mut #component| #component::app_logic(__quoin_app),
                #window_opts,
                __quoin_runtime,
            )
            .run_in(::xilem::EventLoop::with_user_event())
        }
    }
}
