use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::ComponentAst;

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

    let action_methods = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let block = &func.block;
        quote! { #sig #block }
    });

    let render_stmts = &ast.render.stmts;

    let expanded = quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        #vis struct #name {
            props: #props_name,
            #(#state_fields),*
        }

        impl #name {
            pub fn new(cx: &mut gpui::Context<Self>, ctx: quoin_gpui::GpuiContext, props: #props_name) -> Self {
                use quoin::ReactiveContext;
                #(#state_inits)*
                Self {
                    props,
                    #(#state_field_assignments,)*
                }
            }

            #(#action_methods)*
        }

        impl gpui::Render for #name {
            fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                use gpui::*;
                #(#render_stmts)*
            }
        }
    };

    expanded
}
