use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
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
            let #fname = ctx.create_signal(#default);
        }
    });

    let action_methods = &ast.actions;
    let render_body = &ast.render;

    quote! {
        #[derive(Clone)]
        pub struct #props_name {
            #(#props_fields),*
        }

        #[dioxus::prelude::component]
        pub fn #name(props: #props_name) -> dioxus::prelude::Element {
            let ctx = quoin_dioxus::DioxusContext::new();
            #(#state_inits)*
            #(#action_methods)*

            dioxus::prelude::rsx! {
                #render_body
            }
        }
    }
}
