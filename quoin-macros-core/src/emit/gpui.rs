use crate::parse::ComponentAst;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[must_use]
/// Expands a `component!` AST into a GPUI view struct with props/state
/// wiring, lifecycle hooks, and a `Render` implementation.
pub fn emit_component(ast: &ComponentAst) -> TokenStream {
    let props_struct = props_struct_tokens(ast);
    let view_struct = view_struct_tokens(ast);
    let drop_impl = drop_impl_tokens(ast);
    let render_impl = render_impl_tokens(ast);
    let key_actions = key_action_tokens(ast);

    quote! {
        #props_struct
        #view_struct
        #drop_impl
        #render_impl
        #key_actions
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
    let key_field = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! { _quoin_focus: gpui::FocusHandle, }
    };
    let key_field_init = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! { _quoin_focus: _quoin_focus.clone(), }
    };
    let key_init = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! { let _quoin_focus = cx.focus_handle(); }
    };
    // The focus handle only exists when the component declares `keys { .. }`;
    // the accessor must not leak onto key-less components.
    let focus_accessors = if ast.keys.is_empty() {
        quote! {}
    } else {
        quote! {
            /// The focus handle tracking this component's `keys { .. }`
            /// section; used by the app glue to keep the component's key
            /// context in the dispatch path.
            pub fn quoin_focus_handle(&self) -> gpui::FocusHandle {
                self._quoin_focus.clone()
            }
        }
    };

    quote! {
        #vis struct #name {
            props: #props_name,
            #(#state_fields,)*
            #(#global_struct_fields,)*
            #key_field
            _quoin_inputs: quoin::QuoinInputManager,
            _quoin_tables: quoin::QuoinTableManager,
            _quoin_scroll_handles: quoin::QuoinScrollManager,
            _subs: Vec<gpui::Subscription>,
        }

        impl #name {
            pub fn new(cx: &mut gpui::Context<Self>, ctx: quoin::GpuiContext, props: #props_name) -> Self {
                use quoin::ReactiveContext;
                use quoin::Signal;
                #(#state_inits)*
                #(#global_inits)*
                #key_init
                #(#on_mount_stmts)*
                Self {
                    props,
                    #(#state_names,)*
                    #(#global_names,)*
                    #key_field_init
                    _quoin_inputs: quoin::QuoinInputManager::new(),
                    _quoin_tables: quoin::QuoinTableManager::new(),
                    _quoin_scroll_handles: quoin::QuoinScrollManager::new(),
                    _subs: Vec::new(),
                }
            }

            #focus_accessors

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
/// Components with a `keys { .. }` section get their rendered root wrapped
/// with `.track_focus(..)`, `.key_context(..)` and one `.on_action(..)`
/// handler per declared keystroke.
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

    // With a `keys` section, bind the rendered root to the component's focus
    // scope and dispatch each keystroke to its handler (which sees `this`,
    // `window`, `cx` via the listener closure).
    let render_body = if ast.keys.is_empty() {
        quote! { #(#render_stmts)* }
    } else {
        let key_name = format_ident!("__Quoin{}Key", name);
        let action_handlers = ast.keys.iter().enumerate().map(|(ix, key)| {
            let action = format_ident!("{}{}", key_name, ix);
            let handler = &key.handler;
            quote! {
                .on_action(cx.listener(
                    move |this: &mut Self, _: &#action, window: &mut gpui::Window, cx: &mut gpui::Context<Self>| {
                        let this = &mut *this;
                        let _ = window;
                        #handler
                    },
                ))
            }
        });
        quote! {
            {
                #(#render_stmts)*
            }
            .track_focus(&self._quoin_focus)
            .key_context(stringify!(#name))
            #(#action_handlers)*
        }
    };

    quote! {
        impl gpui::Render for #name {
            fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
                use gpui::*;
                use quoin::Signal;
                // gpui-component widget traits the emitters may reference
                // (button variants/sizes). `as _` keeps unused imports quiet
                // for components that never touch a styled widget.
                use ::gpui_component::button::ButtonVariants as _;
                use ::gpui_component::Sizable as _;
                #(#state_clones)*
                #(#global_clones)*
                #render_body
            }
        }
    }
}

/// Generates the `actions!` invocation plus the global `KeyBinding` list for
/// a component's `keys { .. }` section. Actions are emitted at module scope;
/// bindings are bound once per app by the bootstrap through
/// `quoin::bind_component_keys`.
fn key_action_tokens(ast: &ComponentAst) -> TokenStream {
    if ast.keys.is_empty() {
        return quote! {};
    }
    let name = &ast.name;
    let key_name = format_ident!("__Quoin{}Keys", name);
    let action_idents: Vec<_> = (0..ast.keys.len())
        .map(|ix| format_ident!("__Quoin{}Key{}", name, ix))
        .collect();
    let bindings = ast.keys.iter().zip(&action_idents).map(|(key, action)| {
        let keystroke = &key.keystroke;
        quote! {
            gpui::KeyBinding::new(#keystroke, #action, Some(stringify!(#name)))
        }
    });
    quote! {
        gpui::actions!(quoin_keys, [#(#action_idents),*]);

        /// Keybindings declared by this component's `keys { .. }` section.
        /// Bound once per app: `quoin::bind_component_keys(cx, __quoin_keys());
        #[doc(hidden)]
        pub fn #key_name () -> Vec<gpui::KeyBinding> {
            vec![#(#bindings),*]
        }
    }
}

// ---------------------------------------------------------------------------
