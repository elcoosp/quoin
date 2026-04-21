use proc_macro2::TokenStream;
use quote::quote;

#[cfg(feature = "gpui")]
pub fn generate_gpui_virtual_list(
    items_expr: &syn::Expr,
    estimated_height: f32,
    id: &str,
    adapter_name: &str,
    item_render: TokenStream,
) -> TokenStream {
    quote! {
        {
            let __items = #items_expr;
            let __item_sizes: Vec<gpui::Size<gpui::Pixels>> = __items.iter()
                .map(|_| gpui::Size {
                    width: gpui::px(300.0),
                    height: gpui::px(#estimated_height)
                })
                .collect();
            gpui_component::v_virtual_list(
                &self.#adapter_name,
                #id,
                __item_sizes,
                move |_view, __visible_range, _window, _cx| {
                    __visible_range.into_iter().map(|__ix| {
                        let __item = &__items[__ix];
                        #item_render
                    }).collect::<Vec<_>>()
                },
            )
        }
    }
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_virtual_list(
    items_expr: &syn::Expr,
    estimated_height: f32,
    item_render: TokenStream,
) -> TokenStream {
    quote! {
        <leptos_virtual_list::VirtualList
            items=move || #items_expr.clone()
            item_size=#estimated_height
            key=|item| item.id
        >
            {move |item| leptos::prelude::view! { #item_render }}
        </leptos_virtual_list::VirtualList>
    }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_virtual_list(
    items_expr: &syn::Expr,
    estimated_height: f32,
    item_render: TokenStream,
) -> TokenStream {
    quote! {
        dioxus_primitives::RecycleList {
            items: #items_expr.clone(),
            item_height: #estimated_height,
            render: move |item| rsx! { #item_render }
        }
    }
}
