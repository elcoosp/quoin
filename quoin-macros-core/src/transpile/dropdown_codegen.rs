#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;

pub struct MenuItemDef {
    pub label: syn::Expr,
    pub on_click: syn::Expr,
}

#[cfg(feature = "gpui")]
pub fn generate_gpui_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let items: Vec<TokenStream> = menu_items.iter().map(|item| {
        let label = &item.label;
        let on_click = &item.on_click;
        let idents = crate::transpile::collect_handler_idents(on_click);
        let shadows: Vec<TokenStream> = idents.iter().map(|id| {
            quote! { let #id = #id.clone(); }
        }).collect();
        let handler = crate::transpile::strip_move_from_closure(on_click);
        quote! {
            ::gpui::div()
                .px(::gpui::px(12.0))
                .py(::gpui::px(6.0))
                .cursor_pointer()
                .text_color(::gpui::white())
                .hover(|s| s.bg(::gpui::rgb(0x4e4e4e)))
                .child(#label)
                .on_mouse_down(::gpui::MouseButton::Left, {
                    #(#shadows)*
                    let __handler = ::std::rc::Rc::new(#handler);
                    move |_, _, _| { __handler(()); }
                })
                .into_any_element()
        }
    }).collect();

    quote! {
        {
            let __trigger = #trigger_expr;
            ::gpui::div()
                .relative()
                .child(__trigger)
                .child(
                    ::gpui::div()
                        .absolute()
                        .top_full()
                        .left_full()
                        .bg(::gpui::rgb(0x1f2937))
                        .border_1()
                        .border_color(::gpui::rgb(0x374151))
                        .rounded_md()
                        .py_1()
                        .flex_col()
                        .children(#items)
                        .into_any_element()
                )
        }
    }
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_dropdown(
    _trigger_expr: &syn::Expr,
    _menu_items: &[MenuItemDef],
) -> TokenStream {
    quote! { ::gpui::div() }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_dropdown(
    _trigger_expr: &syn::Expr,
    _menu_items: &[MenuItemDef],
) -> TokenStream {
    quote! { ::gpui::div() }
}
