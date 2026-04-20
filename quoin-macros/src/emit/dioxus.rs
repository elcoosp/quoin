// quoin-macros/src/emit/dioxus.rs
use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;
    let props_name = quote::format_ident!("{}Props", name);

    let props_fields = ast.props.iter().map(|p| {
        let fname = &p.name;
        let fty = &p.ty;
        quote! { pub #fname: #fty } // <-- Fixed #ty to #fty
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
        quote! {
            let #name = {
                #block
            };
        }
    });

    let render_body = &ast.render;

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        #[dioxus::prelude::component]
        #vis fn #name() -> dioxus::prelude::Element {
            use quoin::ReactiveContext;
            let ctx = dioxus::prelude::use_hook(quoin_dioxus::DioxusContext::new);
            #(#state_inits)*
            #(#action_closures)*

            dioxus::prelude::rsx! {
                #render_body
            }
        }
    }
}
