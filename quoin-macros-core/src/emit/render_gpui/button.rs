use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, handler::wrap_handler, try_transpile_class};

pub(crate) fn emit_button(el: &Element) -> TokenStream {
    let mut chain = quote! {
        ::gpui::div()
            .cursor_pointer()
            .rounded(::gpui::px(6.0))
            .px(::gpui::px(8.0))
            .py(::gpui::px(8.0))
            .flex()
            .items_center()
            .justify_center()
            .text_color(::gpui::white())
    };

    let primary = find_arg_bool(el, "primary");
    let ghost = find_arg_bool(el, "ghost");
    let destructive = find_arg_bool(el, "destructive");

    if primary {
        chain = quote! { #chain.bg(::gpui::rgb(0x2563eb)) };
    } else if destructive {
        chain = quote! { #chain.bg(::gpui::rgb(0xdc2626)) };
    } else if !ghost {
        chain = quote! { #chain.bg(::gpui::rgb(0x4e4e4e)) };
    }

    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        for style in styles.normal {
            chain = quote! { #chain #style };
        }
        if !styles.hover.is_empty() {
            let hover_tokens = styles.hover;
            chain = quote! { #chain.hover(|__s| __s #(#hover_tokens)*) };
        }
    }

    for child in &el.children {
        let child_tokens = emit_render(child);
        chain = quote! { #chain.child(#child_tokens) };
    }

    if let Some(handler_expr) = find_arg_expr(el, "on_click") {
        let wrap = wrap_handler(handler_expr);
        chain = quote! { #chain.on_mouse_down(::gpui::MouseButton::Left, #wrap) };
    }

    chain
}
