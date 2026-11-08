//! Framework-specific code emission for `quoin` macros.
//!
//! This module defines the [`FrameworkEmitter`] trait and the [`Emitter`] struct,
//! which dispatches to the active framework’s implementation.

/// `#[cfg]` attribute helpers shared by all emitters.
pub mod cfg;
/// The machine-checked rendering-parity table (Floem + Xilem columns),
/// shared by the parity-matrix test and `docs/BACKEND-PARITY.md`.
pub mod parity;
/// The canonical shadcn element contract (rich-elements Phase 0): the
/// Tier-1 element set, the canonical `variant:`/`size:` grammar, and the
/// per-framework backing map. See `docs/quoin-shadcn-rich-elements-evaluation.md`.
pub mod shadcn_spec;
// Shared helpers are consumed exclusively by the framework backends; the
// default (no-framework) build would otherwise carry dead code.
#[cfg(any(
    feature = "gpui",
    feature = "leptos",
    feature = "dioxus",
    feature = "floem",
    feature = "xilem"
))]
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
/// Floem backend for `quoin_render!`.
#[cfg(feature = "floem")]
pub mod render_floem;
/// GPUI backend for `quoin_render!`.
#[cfg(feature = "gpui")]
pub mod render_gpui;
/// Leptos backend for `quoin_render!`.
#[cfg(feature = "leptos")]
pub mod render_leptos;
/// Xilem backend for `quoin_render!`.
#[cfg(feature = "xilem")]
pub mod render_xilem;

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
#[cfg(not(any(
    feature = "gpui",
    feature = "leptos",
    feature = "dioxus",
    feature = "floem",
    feature = "xilem"
)))]
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

// Each impl below is additionally gated on *no other* framework feature being
// enabled. The proc-macro build path always compiles exactly one backend, so
// this changes nothing there — but tests that exercise two emitters at once
// (the parity matrix) call the backend modules directly and must not hit
// conflicting `impl FrameworkEmitter for Emitter` blocks.

// ── Default (no feature) implementation ────────────────────────────────────
#[cfg(not(any(
    feature = "gpui",
    feature = "leptos",
    feature = "dioxus",
    feature = "floem",
    feature = "xilem"
)))]
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

#[cfg(all(
    feature = "gpui",
    not(any(
        feature = "leptos",
        feature = "dioxus",
        feature = "floem",
        feature = "xilem"
    ))
))]
mod gpui_impl {
    use super::{Emitter, FrameworkEmitter};
    use crate::emit::gpui::emit_component;
    use crate::emit::render_gpui::emit_render_root;
    use crate::emit::run_app_gpui::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            // The per-macro root: adds the dynamic-class runtime hoist under
            // the `gpui-rsx-classes` feature (plain expansion otherwise).
            emit_render_root(node)
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

#[cfg(all(
    feature = "leptos",
    not(any(
        feature = "gpui",
        feature = "dioxus",
        feature = "floem",
        feature = "xilem"
    ))
))]
mod leptos_impl {
    use super::{Emitter, FrameworkEmitter};
    use crate::emit::leptos::emit_component;
    use crate::emit::render_leptos::emit_render;
    use crate::emit::run_app_leptos::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
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

#[cfg(all(
    feature = "dioxus",
    not(any(
        feature = "gpui",
        feature = "leptos",
        feature = "floem",
        feature = "xilem"
    ))
))]
mod dioxus_impl {
    use super::{Emitter, FrameworkEmitter};
    use crate::emit::dioxus::emit_component;
    use crate::emit::render_dioxus::emit_render;
    use crate::emit::run_app_dioxus::emit_run_app;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
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

#[cfg(all(
    feature = "xilem",
    not(any(
        feature = "gpui",
        feature = "leptos",
        feature = "dioxus",
        feature = "floem"
    ))
))]
mod xilem_impl {
    use super::{Emitter, FrameworkEmitter};
    use crate::emit::render_xilem::emit_render;
    use proc_macro2::TokenStream as TokenStream2;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            crate::emit::render_xilem::component::emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            emit_render(node)
        }
        fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2 {
            // The core-runtime lowering lives in `render_xilem::emit_effect`
            // so expansion tests can call it directly under combined
            // features (the `Emitter` impls exist only per single backend).
            crate::emit::render_xilem::emit_effect(eff)
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            crate::emit::render_xilem::component::emit_run_app(input)
        }
    }
}

#[cfg(all(
    feature = "floem",
    not(any(
        feature = "gpui",
        feature = "leptos",
        feature = "dioxus",
        feature = "xilem"
    ))
))]
mod floem_impl {
    use super::{Emitter, FrameworkEmitter};
    use crate::emit::render_floem::emit_render;
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    impl FrameworkEmitter for Emitter {
        fn emit_component(&self, ast: &crate::parse::ComponentAst) -> TokenStream2 {
            // Floem has no component runtime: a component becomes a plain fn
            // that creates its signals once and returns the view.
            crate::emit::render_floem::component::emit_component(ast)
        }
        fn emit_render(&self, node: &crate::render_ast::RenderNode) -> TokenStream2 {
            emit_render(node)
        }
        fn emit_effect(&self, eff: &crate::effect::Effect) -> TokenStream2 {
            let body = &eff.body;
            // The `run` field is a closure; it must actually be invoked —
            // inlining it as a statement would evaluate a closure value and
            // silently no-op.
            let run_once = quote! { (#body)(); };
            if let Some(cleanup) = &eff.cleanup {
                // Cleanup: the user's one cleanup closure is installed once
                // in a shared slot (outside the effect, so the `Fn` effect
                // closure never has to move it); the effect runs it before
                // every re-run, and a slot clone owned by a scope-bound
                // signal runs it when the owning scope is disposed (floem
                // has no on_cleanup hook; see quoin::floem_effect).
                quote! {
                    {
                        let __quoin_slot = ::quoin::floem_effect::CleanupSlot::new();
                        __quoin_slot.set(
                            Box::new(#cleanup) as Box<dyn ::core::ops::Fn() + 'static>
                        );
                        ::floem::reactive::RwSignal::new(::core::clone::Clone::clone(
                            &__quoin_slot,
                        ));
                        ::floem::reactive::Effect::new({
                            let __quoin_slot =
                                ::core::clone::Clone::clone(&__quoin_slot);
                            move |__prev: ::core::option::Option<()>| {
                                // The initial run has no previous state to
                                // tear down; every re-run cleans up first.
                                if __prev.is_some() {
                                    __quoin_slot.run();
                                }
                                #run_once
                            }
                        });
                    }
                }
            } else {
                quote! {
                    ::floem::reactive::Effect::new(
                        move |__prev: ::core::option::Option<()>| {
                            let _ = __prev;
                            #run_once
                        },
                    );
                }
            }
        }
        fn emit_run_app(&self, input: &crate::run_app::RunAppInput) -> TokenStream2 {
            crate::emit::render_floem::component::emit_run_app(input)
        }
    }
}
