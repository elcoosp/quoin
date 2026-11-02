use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, handler::wrap_handler, try_transpile_class};

/// Emits a `button(...)` element as a real `gpui_component::Button`.
///
/// Supported framework-agnostic arguments:
/// - `label: &str` / `icon: "icon-name"` / children as the button content
/// - `primary: bool`, `ghost: bool`, `destructive: bool` variants
/// - `small: bool`, `xsmall: bool` sizes (default medium)
/// - `tooltip: &str`
/// - `on_click: |…| …` handler (wrapped, signals shadow-cloned)
/// - `class: "…"` tailwind classes applied on top of the widget defaults
pub(crate) fn emit_button(el: &Element) -> TokenStream {
    let mut chain = quote! { ::gpui_component::button::Button::new("button") };

    if find_arg_bool(el, "primary") {
        chain = quote! { #chain.primary() };
    } else if find_arg_bool(el, "destructive") {
        chain = quote! { #chain.destructive() };
    } else if find_arg_bool(el, "ghost") {
        chain = quote! { #chain.ghost() };
    }

    if find_arg_bool(el, "xsmall") {
        chain = quote! { #chain.xsmall() };
    } else if find_arg_bool(el, "small") {
        chain = quote! { #chain.small() };
    }

    if let Some(icon) = find_arg_string(el, "icon") {
        let resolved = crate::transpile::icon_codegen::icon_variant_ident(&icon);
        chain = quote! { #chain.icon(::gpui_component::Icon::new(::gpui_component::IconName::#resolved)) };
    }

    if let Some(label) = find_arg_string(el, "label") {
        chain = quote! { #chain.label(#label) };
    }

    if let Some(tooltip) = find_arg_string(el, "tooltip") {
        chain = quote! { #chain.tooltip(#tooltip) };
    }

    for child in &el.children {
        let child_tokens = emit_render(child);
        chain = quote! { #chain.child(#child_tokens) };
    }

    if let Some(handler_expr) = find_arg_expr(el, "on_click") {
        let wrap = wrap_handler(handler_expr);
        chain = quote! { #chain.on_click(#wrap) };
    }

    let mut wrapped = quote! { #chain };
    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        for style in styles.normal {
            wrapped = quote! { #wrapped #style };
        }
        if !styles.hover.is_empty() {
            let hover_tokens = styles.hover;
            wrapped = quote! { #wrapped.hover(|__s| __s #(#hover_tokens)*) };
        }
    }

    wrapped
}
