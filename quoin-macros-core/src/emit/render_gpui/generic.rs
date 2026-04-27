use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, handler::wrap_handler, try_transpile_class};

pub(crate) fn emit_generic_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let mut chain = match name_str.as_str() {
        "div" => quote! { ::gpui::div() },
        "h1" => quote! { ::gpui::div().text_xl().font_weight(::gpui::FontWeight::BOLD) },
        "h2" => quote! { ::gpui::div().text_lg().font_weight(::gpui::FontWeight::BOLD) },
        "p" | "text" => quote! { ::gpui::div() },
        _ => quote! { ::gpui::div() },
    };

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

    if let Some(children_expr) = &el.children_expr {
        chain = quote! { #chain.children(#children_expr) };
    } else {
        for child in &el.children {
            let child_tokens = emit_render(child);
            chain = quote! { #chain.child(#child_tokens) };
        }
    }

    if let Some(handler_expr) = find_arg_expr(el, "on_click") {
        let wrap = wrap_handler(handler_expr);
        chain = quote! { #chain.on_mouse_down(::gpui::MouseButton::Left, #wrap) };
    }

    if let Some(handler_expr) = find_arg_expr(el, "on_mouse_down") {
        let wrap = wrap_handler(handler_expr);
        chain = quote! { #chain.on_mouse_down(::gpui::MouseButton::Left, #wrap) };
    }

    chain
}
