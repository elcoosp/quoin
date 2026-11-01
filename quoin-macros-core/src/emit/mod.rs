//! Framework-specific code emission for `quoin` macros.
//!
//! This module defines the [`FrameworkEmitter`] trait and the [`Emitter`] struct,
//! which dispatches to the active framework’s implementation.

/// `#[cfg]` attribute helpers shared by all emitters.
pub mod cfg;
// Shared helpers are consumed exclusively by the framework backends; the
// default (no-framework) build would otherwise carry dead code.
#[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus"))]
pub mod common;

#[cfg(feature = "dioxus")]
/// Dioxus backend for `component!` and `effect!`.
#[cfg(feature = "dioxus")]
pub mod dioxus;
/// GPUI backend for `component!` and `effect!`.
#[cfg(feature = "gpui")]
pub mod gpui;
/// Leptos backend for `component!` and `effect!`.
#[cfg(feature = "leptos")]
pub mod leptos;

/// Dioxus backend for `quoin_render!`.
#[cfg(feature = "dioxus")]
pub mod render_dioxus;
/// GPUI backend for `quoin_render!`.
#[cfg(feature = "gpui")]
pub mod render_gpui;
/// Leptos backend for `quoin_render!`.
#[cfg(feature = "leptos")]
pub mod render_leptos;

/// Dioxus backend for `run_app!`.
#[cfg(feature = "dioxus")]
pub mod run_app_dioxus;
/// GPUI backend for `run_app!`.
#[cfg(feature = "gpui")]
pub mod run_app_gpui;
/// Leptos backend for `run_app!`.
#[cfg(feature = "leptos")]
pub mod run_app_leptos;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── Trait ────────────────────────────────────────────────────────────────────
/// Per-framework expansion interface: one implementation per supported
/// framework feature, selected by [`Emitter`].
pub trait FrameworkEmitter {
    /// Expands a `component!` invocation for the target framework.
    fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2;
    /// Expands a `quoin_render!` invocation for the target framework.
    fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2;
    /// Expands an `effect!` invocation for the target framework.
    fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2;
    /// Expands a `run_app!` invocation for the target framework.
    fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2;
}

// ── Emitter struct ───────────────────────────────────────────────────────────
/// Dispatches to the implementation selected by the enabled framework feature.
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
    use super::{Emitter, FrameworkEmitter, quote};
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
            if eff.cleanup.is_some() {
                quote! { compile_error!("effect! cleanup is not supported in GPUI.") }
            } else {
                quote! { (#body)(); }
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}

#[cfg(feature = "leptos")]
mod leptos_impl {
    use super::{Emitter, FrameworkEmitter, quote};
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
            if let Some(cleanup_expr) = &eff.cleanup {
                quote! {
                    leptos::prelude::create_effect(move |_| { #body; });
                    leptos::prelude::on_cleanup(move || { #cleanup_expr; });
                }
            } else {
                quote! {
                    leptos::prelude::create_effect(move |_| { #body; });
                }
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}

#[cfg(feature = "dioxus")]
mod dioxus_impl {
    use super::{Emitter, FrameworkEmitter, quote};
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
            if let Some(cleanup_expr) = &eff.cleanup {
                quote! {
                    dioxus::prelude::use_effect(move || { #body; });
                    dioxus::prelude::use_drop(move || { #cleanup_expr; });
                }
            } else {
                quote! {
                    dioxus::prelude::use_effect(move || { #body; });
                }
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            emit_run_app(input)
        }
    }
}
