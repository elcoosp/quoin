// quoin-macros/src/emit/dioxus.rs
use crate::parse::ComponentAst;
use crate::transpile::collect_block_idents;
use proc_macro2::TokenStream;
use quote::quote;

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

    let action_closures = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let name = &sig.ident;
        let block = &func.block;
        let referenced = collect_block_idents(block);
        let shadows = referenced.iter().map(|id| {
            quote! { let #id = #id.clone(); }
        });
        quote! {
            let #name = {
                #(#shadows)*
                move || #block
            };
        }
    });

    // Phase 1-D: on_mount → use_effect
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

    // Phase 1-D: on_unmount → use_drop
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

    let global_inits: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! {
            let #fname: Option<quoin::DioxusSignal<#fty>> = ctx.use_global::<#fty>();
        }
    }).collect::<Vec<_>>();

    let render_body = &ast.render;

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        #[dioxus::prelude::component]
        #vis fn #name() -> dioxus::prelude::Element {
            use quoin::ReactiveContext;
            let ctx = dioxus::prelude::use_hook(quoin::DioxusContext::new);
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            // Phase 1-D: on_mount
            #on_mount_tokens
            // Phase 1-D: on_unmount
            #on_unmount_tokens

            dioxus::prelude::rsx! {
                #render_body
            }
        }
    }
}
