#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;

#[cfg(feature = "gpui")]
pub fn generate_gpui_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _id: &str,
    _item_render: TokenStream,
) -> TokenStream {
    // v_virtual_list requires Entity<V> which can't be created inline in quoin_render!
    // Using a simple scrollable div as fallback
    quote! {
        ::gpui::div()
            .size_full()
            .overflow_y_scroll()
    }
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _item_render: TokenStream,
) -> TokenStream {
    quote! { ::gpui::div() }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_virtual_list(
    _items_expr: &syn::Expr,
    _estimated_height: f32,
    _item_render: TokenStream,
) -> TokenStream {
    quote! { ::gpui::div() }
}
