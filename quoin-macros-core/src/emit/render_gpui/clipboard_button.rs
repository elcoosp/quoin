use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::try_transpile_class;

/// Emits a `clipboard_button(...)` element as a real
/// `gpui_component::clipboard::Clipboard` (icon-only copy widget bound to the
/// system clipboard).
///
/// Supported framework-agnostic arguments:
/// - `id: &str` — element id (defaults to `clipboard`)
/// - `value: <expr>` — text to copy when clicked (required)
/// - `tooltip: &str`
/// - `class: "…"` — extra classes
pub(crate) fn emit_clipboard_button(el: &Element) -> TokenStream {
    let Some(copy_text) = find_arg_expr(el, "value").or(find_arg_expr(el, "copy_text")) else {
        return quote! { ::gpui::div().child("clipboard_button: missing value") };
    };
    let id = find_arg_string(el, "id").unwrap_or_else(|| "clipboard".to_string());

    let mut chain = quote! {
        ::gpui_component::clipboard::Clipboard::new(#id).value(#copy_text)
    };

    if let Some(tooltip) = find_arg_string(el, "tooltip") {
        chain = quote! { #chain.tooltip(#tooltip) };
    }

    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        for style in styles.normal {
            chain = quote! { #chain #style };
        }
    }

    quote! { #chain }
}
