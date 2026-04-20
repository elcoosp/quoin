use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let name = &ast.name;

    // Props become function arguments
    let prop_args = ast.props.iter().map(|p| {
        let fname = &p.name;
        let fty = &p.ty;
        quote! { #fname: #fty }
    });

    // State becomes signals created inside the component function
    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = ctx.create_signal(#default);
        }
    });

    // Action methods become nested functions
    let action_methods = &ast.actions;

    // Render body is wrapped in view! macro
    let render_body = &ast.render;

    let expanded = quote! {
        #[leptos::prelude::component]
        pub fn #name(#(#prop_args),*) -> impl leptos::prelude::IntoView {
            let ctx = quoin_leptos::LeptosContext::new();
            #(#state_inits)*
            #(#action_methods)*

            leptos::prelude::view! {
                #render_body
            }
        }
    };

    expanded
}
