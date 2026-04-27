//! Framework-specific code emission for `quoin` macros.
//!
//! This module defines the [`FrameworkEmitter`] trait and the [`Emitter`] struct,
//! which dispatches to the active framework’s implementation.

pub mod cfg;
pub mod common;

#[cfg(feature = "dioxus")]
pub mod dioxus;
#[cfg(feature = "gpui")]
pub mod gpui;
#[cfg(feature = "leptos")]
pub mod leptos;

#[cfg(feature = "dioxus")]
pub mod render_dioxus;
#[cfg(feature = "gpui")]
pub mod render_gpui;
#[cfg(feature = "leptos")]
pub mod render_leptos;

#[cfg(feature = "dioxus")]
pub mod run_app_dioxus;
#[cfg(feature = "gpui")]
pub mod run_app_gpui;
#[cfg(feature = "leptos")]
pub mod run_app_leptos;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── Trait ────────────────────────────────────────────────────────────────────
pub trait FrameworkEmitter {
    fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2;
    fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2;
    fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2;
    fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2;
}

// ── Emitter struct ───────────────────────────────────────────────────────────
pub struct Emitter;

// ── Default (no feature) implementation ────────────────────────────────────
#[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
impl FrameworkEmitter for Emitter {
    fn emit_component(&self, _: &crate::parse::ComponentAst) -> TokenStream2 {
        quote! { compile_error!("component! requires a framework feature"); }
    }
    fn emit_render(&self, _: &crate::render_ast::RenderNode) -> TokenStream2 {
        quote! { compile_error!("quoin_render! requires a framework feature"); }
    }
    fn emit_effect(&self, _: &crate::effect::Effect) -> TokenStream2 {
        quote! { compile_error!("effect! requires a framework feature"); }
    }
    fn emit_run_app(&self, _: &crate::run_app::RunAppInput) -> TokenStream2 {
        quote! { compile_error!("run_app! requires a framework feature"); }
    }
}

#[cfg(feature = "gpui")]
mod gpui_impl {
    use super::*;
    use crate::emit::gpui::emit_component;
    use crate::emit::render_gpui::emit_render;
    use crate::emit::run_app_gpui::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            emit_render(node)
        }
        fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2 {
            let body = &eff.body;
            match &eff.cleanup {
                Some(_) => quote! { compile_error!("effect! cleanup is not supported in GPUI.") },
                None => quote! { (#body)(); },
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}

#[cfg(feature = "leptos")]
mod leptos_impl {
    use super::*;
    use crate::emit::leptos::emit_component;
    use crate::emit::render_leptos::emit_render;
    use crate::emit::run_app_leptos::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            emit_render(node)
        }
        fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2 {
            let body = &eff.body;
            match &eff.cleanup {
                Some(cleanup_expr) => {
                    quote! {
                        leptos::prelude::create_effect(move |_| { #body; });
                        leptos::prelude::on_cleanup(move || { #cleanup_expr; });
                    }
                }
                None => quote! {
                    leptos::prelude::create_effect(move |_| { #body; });
                },
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}

#[cfg(feature = "dioxus")]
mod dioxus_impl {
    use super::*;
    use crate::emit::dioxus::emit_component;
    use crate::emit::render_dioxus::emit_render;
    use crate::emit::run_app_dioxus::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            emit_render(node)
        }
        fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2 {
            let body = &eff.body;
            match &eff.cleanup {
                Some(cleanup_expr) => {
                    quote! {
                        dioxus::prelude::use_effect(move || { #body; });
                        dioxus::prelude::use_drop(move || { #cleanup_expr; });
                    }
                }
                None => quote! {
                    dioxus::prelude::use_effect(move || { #body; });
                },
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}
