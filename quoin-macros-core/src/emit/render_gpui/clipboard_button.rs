use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, try_transpile_class};

pub(crate) fn emit_clipboard_button(el: &Element) -> TokenStream {
    let copy_text = match find_arg_expr(el, "copy_text") {
        Some(e) => e,
        None => return quote! { ::gpui::div().child("clipboard_button: missing copy_text") },
    };

    let mut chain = quote! {
        ::gpui::div().cursor_pointer().rounded(::gpui::px(6.0)).px(::gpui::px(8.0)).py(::gpui::px(8.0))
            .flex().items_center().justify_center().text_color(::gpui::white()).bg(::gpui::rgb(0x4e4e4e))
    };

    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        for style in styles.normal {
            chain = quote! { #chain #style };
        }
    }

    for child in &el.children {
        let child_tokens = emit_render(child);
        chain = quote! { #chain.child(#child_tokens) };
    }

    let copy_text_clone = copy_text.clone();
    chain = quote! {
        #chain.on_mouse_down(::gpui::MouseButton::Left,
            move |_, _, cx| {
                cx.write_to_clipboard(::gpui::ClipboardItem::new_string(#copy_text_clone.to_string()));
            }
        )
    };

    chain
}
