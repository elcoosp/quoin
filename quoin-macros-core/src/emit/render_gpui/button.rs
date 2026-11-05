use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, handler::wrap_handler, try_transpile_class};

/// Emits a `button(...)` element as a real `gpui_component::Button`.
///
/// Supported framework-agnostic arguments:
/// - `label: &str` / `icon: "icon-name"` / children as the button content
/// - `variant: "default"|"primary"|"destructive"|"outline"|"secondary"|"ghost"|"link"`
///   (canonical shadcn spelling — see `emit::shadcn_spec`), with the
///   boolean sugar `primary:`/`ghost:`/`destructive:` still accepted
/// - `small: bool`, `xsmall: bool` sizes, or canonical `size: "sm"|"lg"`
/// - `tooltip: &str`
/// - `on_click: |…| …` handler (wrapped, signals shadow-cloned)
/// - `class: "…"` tailwind classes applied on top of the widget defaults
pub(crate) fn emit_button(el: &Element) -> TokenStream {
    let mut chain = quote! { ::gpui_component::button::Button::new("button") };

    // Canonical `variant:` first, boolean sugar as the fallback. Unknown
    // `variant:` values are a hard error (silence is never an option);
    // `outline`/`secondary`/`link` map onto the matching gpui-component
    // variants. The canonical contract lives in `emit::shadcn_spec`.
    if let Some(choice) = crate::emit::shadcn_spec::canonical_button_variant(el) {
        let error = crate::emit::shadcn_spec::choice_error(&choice);
        if let Some(error) = error {
            return error;
        }
        chain = match choice.unwrap() {
            "default" | "primary" => quote! { #chain.primary() },
            // gpui-component names the red variant `danger` (the canonical
            // shadcn `destructive` renames land with gpui-kit #2311).
            "destructive" => quote! { #chain.danger() },
            "ghost" => quote! { #chain.ghost() },
            "outline" => quote! { #chain.outline() },
            "secondary" => quote! { #chain.secondary() },
            "link" => quote! { #chain.link() },
            // `canonical_button_variant` only yields `BUTTON_VARIANTS`.
            _ => chain,
        };
    } else if find_arg_bool(el, "primary") {
        chain = quote! { #chain.primary() };
    } else if find_arg_bool(el, "destructive") {
        // gpui-component names the red variant `danger` (see above).
        chain = quote! { #chain.danger() };
    } else if find_arg_bool(el, "ghost") {
        chain = quote! { #chain.ghost() };
    }

    // Canonical `size:` alongside the historical boolean sizes.
    if let Some(choice) = crate::emit::shadcn_spec::canonical_button_size(el) {
        let error = crate::emit::shadcn_spec::choice_error(&choice);
        if let Some(error) = error {
            return error;
        }
        chain = match choice.unwrap() {
            "sm" => quote! { #chain.small() },
            "lg" => quote! { #chain.large() },
            // `icon` sizing stays a gpui-component-native concern for now
            // (`xsmall` keeps the compact feel without the square lock).
            "icon" => quote! { #chain.xsmall() },
            _ => chain,
        };
    } else if find_arg_bool(el, "xsmall") {
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
        wrapped = crate::transpile::tailwind::apply_normal_and_hover(wrapped, &styles);
    }

    wrapped
}
