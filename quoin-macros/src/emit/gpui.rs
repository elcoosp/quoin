use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;
    let props_name = quote::format_ident!("{}Props", name);

    let props_fields: Vec<_> = ast
        .props
        .iter()
        .map(|p| {
            let fname = &p.name;
            let fty = &p.ty;
            quote! { pub #fname: #fty }
        })
        .collect();

    let state_fields: Vec<_> = ast
        .state
        .iter()
        .map(|s| {
            let fname = &s.name;
            let sty = &s.ty;
            quote! { #fname: quoin_gpui::GpuiSignal<#sty> }
        })
        .collect();

    let state_inits: Vec<_> = ast
        .state
        .iter()
        .map(|s| {
            let fname = &s.name;
            let default = &s.default;
            quote! {
                let #fname = ctx.create_signal(#default);
            }
        })
        .collect();

    let state_field_assignments: Vec<_> = ast
        .state
        .iter()
        .map(|s| {
            let fname = &s.name;
            quote! { #fname }
        })
        .collect();

    let state_clones: Vec<_> = ast
        .state
        .iter()
        .map(|s| {
            let fname = &s.name;
            quote! { let #fname = self.#fname.clone(); }
        })
        .collect();

    let action_methods: Vec<_> = ast
        .actions
        .iter()
        .map(|func| {
            let sig = &func.sig;
            let block = &func.block;
            quote! { #sig #block }
        })
        .collect();

    let render_stmts = &ast.render.stmts;

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
        }

        impl Default for #props_name {
            fn default() -> Self {
                Self {
                    #(
                        #props_fields: Default::default()
                    ),*
                }
            }
        }

        #vis struct #name {
            props: #props_name,
            #(#state_fields,)* // FIX 1: Comma inside repetition
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
                #(#state_clones)*
                #(#render_stmts)*
            }
        }
    }
}
