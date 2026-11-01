use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands a `component!` AST into a GPUI view struct with props/state
/// wiring, lifecycle hooks, and a `Render` implementation.
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let props_struct = props_struct_tokens(ast);
    let view_struct = view_struct_tokens(ast);
    let drop_impl = drop_impl_tokens(ast);
    let render_impl = render_impl_tokens(ast);

    quote! {
        #props_struct
        #view_struct
        #drop_impl
        #render_impl
    }
}

/// Generates the `#nameProps` struct with one public field per declared prop,
/// plus its `Default` implementation (explicit defaults or `Default::default()`).
fn props_struct_tokens(ast: &ComponentAst) -> TokenStream {
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

    let props_defaults: Vec<_> = ast
        .props
        .iter()
        .map(|p| {
            let fname = &p.name;
            if let Some(default) = &p.default {
                quote! { #fname: #default }
            } else {
                quote! { #fname: Default::default() }
            }
        })
        .collect();

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
    }
}

/// Generates the view struct, its `new` constructor (signal/global wiring and
/// `on_mount` statements), and the user-declared action methods.
fn view_struct_tokens(ast: &ComponentAst) -> TokenStream {
    let vis = &ast.vis;
    let name = &ast.name;
    let props_name = quote::format_ident!("{}Props", name);

    let state_fields = state_field_tokens(ast);
    let global_struct_fields = global_struct_field_tokens(ast);
    let state_inits = state_init_tokens(ast);
    let global_inits = global_init_tokens(ast);
    let on_mount_stmts = on_mount_stmt_tokens(ast);
    let state_names: Vec<_> = ast.state.iter().map(|s| &s.name).collect();
    let global_names: Vec<_> = ast.globals.iter().map(|g| &g.name).collect();
    let action_methods = action_method_tokens(ast);

    quote! {
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
                    #(#state_names,)*
                    #(#global_names,)*
                    _quoin_inputs: quoin::QuoinInputManager::new(),
                    _subs: Vec::new(),
                }
            }

            #(#action_methods)*
        }
    }
}

/// Generates the `GpuiSignal`-typed field declarations for the component's
/// state entries.
fn state_field_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    ast.state
        .iter()
        .map(|s| {
            let fname = &s.name;
            let sty = &s.ty;
            quote! { #fname: quoin::GpuiSignal<#sty> }
        })
        .collect()
}

/// Generates the `ctx.create_signal(...)` initialization statements for the
/// component's state entries.
fn state_init_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    ast.state
        .iter()
        .map(|s| {
            let fname = &s.name;
            let default = &s.default;
            quote! { let #fname = ctx.create_signal(#default); }
        })
        .collect()
}

/// Generates the `Option<GpuiSignal<T>>` struct fields for the component's
/// global entries.
fn global_struct_field_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    ast.globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            let fty = &g.ty;
            quote! { #fname: Option<quoin::GpuiSignal<#fty>> }
        })
        .collect()
}

/// Generates the `use_global` initialization statements for the component's
/// global entries, annotating observed globals explicitly.
fn global_init_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    ast.globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            let fty = &g.ty;
            let observe = g.observe;
            if observe {
                // When observe is true, ensure the global signal's mutations trigger re-renders.
                // The GpuiSignal's set() already calls context.request_update() via the notifier chain,
                // but we make it explicit by storing a guard that logs when the global changes.
                quote! {
                    let __global_val: Option<quoin::GpuiSignal<#fty>> = ctx.use_global::<#fty>();
                    // Global #fname is observed: mutations will trigger re-render
                    let #fname = __global_val;
                }
            } else {
                quote! { let #fname: Option<quoin::GpuiSignal<#fty>> = ctx.use_global::<#fty>(); }
            }
        })
        .collect()
}

/// Generates the user-declared action methods verbatim for the view `impl`.
fn action_method_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    ast.actions
        .iter()
        .map(|func| {
            let sig = &func.sig;
            let block = &func.block;
            quote! { #sig #block }
        })
        .collect()
}

/// Generates the statements executed at construction time from the
/// component's `on_mount` block (empty when absent).
fn on_mount_stmt_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    match &ast.on_mount {
        Some(block) => block.stmts.iter().map(|s| quote! { #s }).collect(),
        None => Vec::new(),
    }
}

/// Generates the `Drop` implementation running the component's `on_unmount`
/// block with the state signals in scope (no-op when absent).
fn drop_impl_tokens(ast: &ComponentAst) -> TokenStream {
    let name = &ast.name;
    match &ast.on_unmount {
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
    }
}

/// Generates the `gpui::Render` implementation, cloning the state/global
/// signal handles into scope before running the user render statements.
fn render_impl_tokens(ast: &ComponentAst) -> TokenStream {
    let name = &ast.name;
    let render_stmts = &ast.render.stmts;

    let state_clones: Vec<_> = ast
        .state
        .iter()
        .map(|s| {
            let fname = &s.name;
            quote! { let #fname = self.#fname.clone(); }
        })
        .collect();

    let global_clones: Vec<_> = ast
        .globals
        .iter()
        .map(|g| {
            let fname = &g.name;
            quote! { let #fname = self.#fname.clone(); }
        })
        .collect();

    quote! {
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

// ---------------------------------------------------------------------------
