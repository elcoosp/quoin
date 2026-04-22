use crate::parse::ComponentAst;
use crate::transpile::collect_block_idents;
use proc_macro2::TokenStream;
use quote::quote;

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

    let state_global_names: std::collections::HashSet<String> = ast
        .state
        .iter()
        .map(|s| s.name.to_string())
        .chain(ast.globals.iter().map(|g| g.name.to_string()))
        .collect();

    let action_closures = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let name = &sig.ident;
        let block = &func.block;

        let param_names: std::collections::HashSet<String> = sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg
                    && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                {
                    return Some(pat_ident.ident.to_string());
                }
                None
            })
            .collect();

        let params: Vec<(proc_macro2::Ident, &syn::Type)> = sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg
                    && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                {
                    return Some((pat_ident.ident.clone(), &*pat_type.ty));
                }
                None
            })
            .collect();

        let referenced = collect_block_idents(block);
        let shadows: Vec<_> = referenced
            .iter()
            .filter(|id| {
                let name_str = id.to_string();
                state_global_names.contains(&name_str) && !param_names.contains(&name_str)
            })
            .map(|id| {
                quote! { let #id = #id.clone(); }
            })
            .collect();

        if params.is_empty() {
            quote! {
                let #name = {
                    #(#shadows)*
                    move || #block
                };
            }
        } else {
            let param_idents: Vec<_> = params.iter().map(|(id, _)| id).collect();
            let param_types: Vec<_> = params.iter().map(|(_, ty)| ty).collect();
            quote! {
                let #name = {
                    #(#shadows)*
                    move |#(#param_idents: #param_types),*| #block
                };
            }
        }
    });

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
            #(#on_mount_tokens)*
            #on_unmount_tokens

            #(#render_stmts)*
        }
    }
}
