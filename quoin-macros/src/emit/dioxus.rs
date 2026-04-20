use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let name = &ast.name;

    // Props as function arguments
    let prop_args = ast.props.iter().map(|p| {
        let fname = &p.name;
        let fty = &p.ty;
        quote! { #fname: #fty }
    });

    // State signals created via context
    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = ctx.create_signal(#default);
        }
    });

    let action_methods = &ast.actions;
    let render_body = &ast.render;

    quote! {
        #[dioxus::prelude::component]
        pub fn #name(#(#prop_args),*) -> dioxus::prelude::Element {
            let ctx = quoin_dioxus::DioxusContext::new();
            #(#state_inits)*
            #(#action_methods)*

            dioxus::prelude::rsx! {
                #render_body
            }
        }
    }
}
