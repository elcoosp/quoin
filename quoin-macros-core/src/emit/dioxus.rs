use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST into a Dioxus component function with hook-based
/// signal initialization, action closures, lifecycle hooks, and a render body
/// returning an `Element`.
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;
    let props_name = quote::format_ident!("{}Props", name);

    let props_fields = ast.props.iter().map(|p| {
        let fname = &p.name;
        let fty = &p.ty;
        quote! { pub #fname: #fty }
    });

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = dioxus::prelude::use_hook(|| ctx.create_signal(#default));
        }
    });

    let action_closures = crate::emit::common::action_closure_tokens(ast);

    let on_mount_tokens: TokenStream = match &ast.on_mount {
        Some(block) => {
            let stmts: Vec<TokenStream> = block.stmts.iter().map(|s| quote! { #s }).collect();
            quote! {
                dioxus::prelude::use_effect(move || {
                    #(#stmts)*
                });
            }
        }
        None => quote! {},
    };

    let on_unmount_tokens: TokenStream = match &ast.on_unmount {
        Some(block) => {
            let stmts: Vec<TokenStream> = block.stmts.iter().map(|s| quote! { #s }).collect();
            quote! {
                dioxus::prelude::use_drop(move || {
                    #(#stmts)*
                });
            }
        }
        None => quote! {},
    };

    let global_inits: Vec<_> = ast
        .globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            let fty = &g.ty;
            quote! {
                let #fname: Option<quoin::DioxusSignal<#fty>> = ctx.use_global::<#fty>();
            }
        })
        .collect::<Vec<_>>();

    // `keys { .. }` has no web routing yet — the documented diagnostic
    // contract (docs/BACKEND-PARITY.md), never a silent drop.
    let keys_error = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! {
            ::core::compile_error!(
                "keys { .. } is not supported on the Dioxus backend (no global key \
                 routing in v1); attach key events on the specific widget instead \
                 (see docs/BACKEND-PARITY.md)"
            );
        }
    };

    let render_stmts = &ast.render.stmts;

    // Dioxus 0.7's #[component] requires the function body to return
    // Result<VNode, RenderError>. If the last render statement is a
    // let-binding (ends with ;), it returns () — we need to append
    // a fallback Element.
    let needs_fallback = ast.render.stmts.last().is_none_or(|last| {
        matches!(last, syn::Stmt::Local(_) | syn::Stmt::Item(_))
            || matches!(last, syn::Stmt::Expr(_, Some(_)))
    });

    let fallback = if needs_fallback {
        quote! { Ok(dioxus::prelude::VNode::placeholder()) }
    } else {
        quote! {}
    };

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        #[dioxus::prelude::component]
        #vis fn #name() -> dioxus::prelude::Element {
            use quoin::ReactiveContext;
            use quoin::Signal;
            #keys_error
            let ctx = dioxus::prelude::use_hook(quoin::DioxusContext::new);
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            #on_mount_tokens
            #on_unmount_tokens

            #(#render_stmts)*
            #fallback
        }
    }
}
// ---------------------------------------------------------------------------
