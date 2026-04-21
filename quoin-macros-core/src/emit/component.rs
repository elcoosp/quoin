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
        quote! { pub #fname: #fty }
    });

    let state_fields = ast.state.iter().map(|s| {
        let fname = &s.name;
        let sty = &s.ty;
        quote! { #fname: quoin_gpui::GpuiSignal<#sty> }
    });

    let state_inits = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! {
            let #fname = ctx.create_signal(#default);
        }
    });

    let state_field_assignments = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { #fname }
    });

    let state_destructure = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { let #fname = &self.#fname; }
    });

    let action_methods = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let block = &func.block;
        quote! { #sig #block }
    });

    let render_stmts = &ast.render.stmts;

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        #vis struct #name {
            props: #props_name,
            #(#state_fields),*
            _quoin_inputs: quoin_ui_gpui::QuoinInputManager,
        }

        impl #name {
            pub fn new(cx: &mut gpui::Context<Self>, ctx: quoin_gpui::GpuiContext, props: #props_name) -> Self {
                use quoin::ReactiveContext;
                #(#state_inits)*
                Self {
                    props,
                    #(#state_field_assignments,)*
                    _quoin_inputs: quoin_ui_gpui::QuoinInputManager::new(),
                }
            }

            #(#action_methods)*
        }

        impl gpui::Render for #name {
            fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                use gpui::*;
                #(#state_destructure)*
                #(#render_stmts)*
            }
        }
    }
}
