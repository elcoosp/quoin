use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let name = &ast.name;

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = ctx.create_signal(#default);
        }
    });

    let action_methods = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let block = &func.block;
        quote! { #sig #block }
    });

    let render_stmts = &ast.render.stmts;

    quote! {
        #[leptos::prelude::component]
        pub fn #name() -> impl leptos::prelude::IntoView {
            let ctx = quoin_leptos::LeptosContext::new();
            #(#state_inits)*
            #(#action_methods)*

            #(#render_stmts)*
        }
    }
}
