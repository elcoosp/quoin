use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;
use crate::transpile::collect_block_idents;

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

    // Phase 1-C: on_mount runs immediately after signal init
    let on_mount_tokens: Vec<TokenStream> = match &ast.on_mount {
        Some(block) => {
            let stmts: Vec<TokenStream> = block.stmts.iter().map(|s| quote! { #s }).collect();
            stmts
        }
        None => Vec::new(),
    };

    // Phase 1-C: on_unmount → on_cleanup
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

    let global_inits: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! {
            let #fname: Option<quoin::LeptosSignal<#fty>> = ctx.use_global::<#fty>();
        }
    }).collect::<Vec<_>>();

    let render_stmts = &ast.render.stmts;

    quote! {
        #[leptos::prelude::component]
        #vis fn #name() -> impl leptos::prelude::IntoView {
            use quoin::ReactiveContext;
            use quoin::Signal;
            use leptos::prelude::ElementChild;
            let ctx = quoin::LeptosContext::new();
            #(#state_inits)*
            #(#global_inits)*
            #(#action_closures)*
            // Phase 1-C: on_mount
            #(#on_mount_tokens)*
            // Phase 1-C: on_unmount
            #on_unmount_tokens

            #(#render_stmts)*
        }
    }
}
