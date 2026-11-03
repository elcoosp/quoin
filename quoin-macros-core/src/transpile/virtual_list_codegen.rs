//! Virtual list (lazy scrolling) code generation.
//!
//! Generates framework-specific code for rendering large lists with
//! viewport-based rendering. Currently all implementations fall back to
//! simple scrollable containers because true virtual scrolling requires
//! framework-specific entity/lifecycle management that cannot be expressed
//! inline in `quoin_render!`:
//!
//! - **GPUI** ([`generate_gpui_virtual_list`]): `v_virtual_list` requires
//!   `Entity<V>`, which cannot be created in a `Render` impl. Falls back to
//!   `div().size_full().overflow_y_scroll()`.
//! - **Leptos / Dioxus**: Emit scrollable divs. True windowing can be added
//!   later via `leptos-virtual-scroll` or `dioxus-primitives` `RecycleList`.

#[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus"))]
use proc_macro2::TokenStream;
#[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus"))]
use quote::quote;

#[cfg(feature = "gpui")]
#[must_use]
/// Generates a scrollable virtual list for GPUI (currently falls back to a
/// plain scrollable div; arguments are reserved for windowing support).
pub fn generate_gpui_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _id: &str,
    _item_render: TokenStream,
) -> TokenStream {
    // v_virtual_list requires Entity<V> which can't be created inline in quoin_render!
    // Using a simple scrollable div as fallback. `overflow_y_scroll` is a
    // `StatefulInteractiveElement` method: the div must be stateful (`.id`)
    // or the method does not exist.
    quote! {
        ::gpui::div()
            .id("quoin-virtual-list")
            .size_full()
            .overflow_y_scroll()
    }
}

#[cfg(feature = "leptos")]
#[must_use]
/// Generates a scrollable virtual list for Leptos (currently falls back to a
/// plain scrollable div; arguments are reserved for windowing support).
pub fn generate_leptos_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _item_render: TokenStream,
) -> TokenStream {
    // TODO: upgrade to leptos-virtual-scroll when estimated_height is provided
    quote! {
        ::leptos::html::div()
            .attr("style", "overflow-y: auto")
    }
}

#[cfg(feature = "dioxus")]
#[must_use]
/// Generates a scrollable virtual list for Dioxus (currently falls back to a
/// plain scrollable div; arguments are reserved for windowing support).
pub fn generate_dioxus_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _item_render: TokenStream,
) -> TokenStream {
    // TODO: upgrade to dioxus-primitives RecycleList when available
    quote! {
        ::dioxus::prelude::rsx! {
            div { style: "overflow-y: auto" }
        }
    }
}
