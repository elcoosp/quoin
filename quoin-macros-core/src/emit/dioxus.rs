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

    // `keys { .. }` lowers natively on Dioxus: one window-level `keydown`
    // listener per component through `quoin::keys::attach_window_keydown`,
    // disposed with the component scope (`use_drop`) — the web counterpart
    // of the Floem one-listener contract (docs/BACKEND-PARITY.md). Native
    // (desktop/SSR) builds compile the same expansion: the attach is the
    // documented no-op there (the browser webview routing is future work).
    let keys_block = crate::emit::web_keys::component_keys_block(
        &ast.keys,
        &crate::emit::web_keys::state_globals_set(
            ast.state.iter().map(|s| s.name.to_string()),
            ast.globals.iter().map(|g| g.name.to_string()),
        ),
        &quote! { ::dioxus::prelude::use_drop(move || __quoin_keys_handle.remove()); },
    );

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
            let ctx = dioxus::prelude::use_hook(quoin::DioxusContext::new);
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            #keys_block
            #on_mount_tokens
            #on_unmount_tokens

            #(#render_stmts)*
            #fallback
        }
    }
}
// ---------------------------------------------------------------------------
