use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

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
        let block = &func.block;
        let name = &sig.ident;
        quote! {
            let #name = || #block;
        }
    });

    let render_stmts = &ast.render.stmts;

    quote! {
        #[leptos::prelude::component]
        #vis fn #name() -> impl leptos::prelude::IntoView {
            use quoin::ReactiveContext;
            use quoin::Signal;
            use leptos::prelude::ElementChild;
            let ctx = quoin_leptos::LeptosContext::new();
            #(#state_inits)*
            #(#action_closures)*

            #(#render_stmts)*
        }
    }
}
