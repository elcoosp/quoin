use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;
    let props_name = quote::format_ident!("{}Props", name);

    let props_fields: Vec<_> = ast.props.iter().map(|p| {
        let fname = &p.name;
        let fty = &p.ty;
        quote! { pub #fname: #fty }
    }).collect();

    let props_defaults: Vec<_> = ast.props.iter().map(|p| {
        let fname = &p.name;
        if let Some(default) = &p.default {
            quote! { #fname: #default }
        } else {
            quote! { #fname: Default::default() }
        }
    }).collect();

    let state_fields: Vec<_> = ast.state.iter().map(|s| {
        let fname = &s.name;
        let sty = &s.ty;
        quote! { #fname: quoin::GpuiSignal<#sty> }
    }).collect();

    let state_inits: Vec<_> = ast.state.iter().map(|s| {
        let fname = &s.name;
        let default = &s.default;
        quote! { let #fname = ctx.create_signal(#default); }
    }).collect();

    let state_field_assignments: Vec<_> = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { #fname }
    }).collect();

    // Global struct fields: name: Option<GpuiSignal<Ty>>
    let global_struct_fields: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! { #fname: Option<quoin::GpuiSignal<#fty>> }
    }).collect();

    // Global init calls: let name = ctx.use_global::<Ty>();
    let global_inits: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        let fty = &g.ty;
        quote! { let #fname: Option<quoin::GpuiSignal<#fty>> = ctx.use_global::<#fty>(); }
    }).collect();

    // Global self field assignments
    let global_self_fields: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        quote! { #fname }
    }).collect();

    let state_clones: Vec<_> = ast.state.iter().map(|s| {
        let fname = &s.name;
        quote! { let #fname = self.#fname.clone(); }
    }).collect();

    // Clone global fields for render access
    let global_clones: Vec<_> = ast.globals.iter().map(|g| {
        let fname = &g.name;
        quote! { let #fname = self.#fname.clone(); }
    }).collect();

    let action_methods: Vec<_> = ast.actions.iter().map(|func| {
        let sig = &func.sig;
        let block = &func.block;
        quote! { #sig #block }
    }).collect();

    let on_mount_stmts: Vec<TokenStream> = match &ast.on_mount {
        Some(block) => block.stmts.iter().map(|s| quote! { #s }).collect(),
        None => Vec::new(),
    };

    let drop_impl = match &ast.on_unmount {
        Some(block) => {
            let stmts: Vec<TokenStream> = block.stmts.iter().map(|s| quote! { #s }).collect();
            let state_names = ast.state.iter().map(|s| &s.name);
            quote! {
                impl Drop for #name {
                    fn drop(&mut self) {
                        use quoin::Signal;
                        #(let #state_names = &self.#state_names;)*
                        #(#stmts)*
                    }
                }
            }
        }
        None => quote! {},
    };

    let render_stmts = &ast.render.stmts;

    quote! {
        #[derive(Clone)]
        #vis struct #props_name {
            #(#props_fields),*
            _phantom: ::std::marker::PhantomData<()>,
        }

        impl Default for #props_name {
            fn default() -> Self {
                Self {
                    #(#props_defaults,)*
                    _phantom: ::std::marker::PhantomData,
                }
            }
        }

        #vis struct #name {
            props: #props_name,
            #(#state_fields,)*
            #(#global_struct_fields,)*
            _quoin_inputs: quoin::QuoinInputManager,
            _subs: Vec<gpui::Subscription>,
        }

        impl #name {
            pub fn new(cx: &mut gpui::Context<Self>, ctx: quoin::GpuiContext, props: #props_name) -> Self {
                use quoin::ReactiveContext;
                use quoin::Signal;
                #(#state_inits)*
                #(#global_inits)*
                #(#on_mount_stmts)*
                Self {
                    props,
                    #(#state_field_assignments,)*
                    #(#global_self_fields,)*
                    _quoin_inputs: quoin::QuoinInputManager::new(),
                    _subs: Vec::new(),
                }
            }

            #(#action_methods)*
        }

        #drop_impl

        impl gpui::Render for #name {
            fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                use gpui::*;
                use quoin::Signal;
                #(#state_clones)*
                #(#global_clones)*
                #(#render_stmts)*
            }
        }
    }
}
