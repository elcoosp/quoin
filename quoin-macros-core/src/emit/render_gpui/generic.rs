use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render, handler::wrap_handler, try_transpile_class};

/// Per-compilation counter giving scroll divs unique element ids.
fn next_scroll_site() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SITE: AtomicUsize = AtomicUsize::new(0);
    SITE.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn emit_generic_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let mut chain = match name_str.as_str() {
        "h1" => quote! { ::gpui::div().text_xl().font_weight(::gpui::FontWeight::BOLD) },
        "h2" => quote! { ::gpui::div().text_lg().font_weight(::gpui::FontWeight::BOLD) },
        // `div`, `p`, `text`, and unknown tags all map to a plain div.
        _ => quote! { ::gpui::div() },
    };

    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        // Scroll overflow classes need a stateful element: make this div
        // stateful before any style applies. The id must be unique per
        // expansion site; a per-compilation counter keeps it deterministic.
        if styles.scroll {
            let site = next_scroll_site();
            let id = format!("quoin-scroll-{site}");
            chain = quote! {
                {
                    use ::gpui::InteractiveElement as _;
                    #chain.id(#id)
                }
            };
        }
        chain = crate::transpile::tailwind::apply_normal_and_hover(chain, &styles);
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

/// Emits an `icon(name: "…")` element as a `gpui_component::Icon`.
pub(crate) fn emit_icon(el: &Element) -> TokenStream {
    let name =
        crate::emit::common::find_arg_string(el, "name").expect("icon requires 'name:' argument");
    let variant = crate::transpile::icon_codegen::icon_variant_ident(&name);
    let mut chain = quote! {
        ::gpui_component::Icon::new(::gpui_component::IconName::#variant)
    };
    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        chain = crate::transpile::tailwind::apply_normal(chain, &styles);
    }
    chain
}

/// Emits a `styled_text(text: …, runs: …)` element. `runs` is a
/// `Vec<quoin::StyledRun>` (framework-agnostic runs of styled text);
/// the GPUI backend resolves tokens to theme colors at runtime.
pub(crate) fn emit_styled_text(el: &Element) -> TokenStream {
    let text_expr = find_arg_expr(el, "text").expect("styled_text requires 'text:' argument");
    let Some(runs_expr) = find_arg_expr(el, "runs") else {
        return quote! { ::gpui::div().child("styled_text: missing runs") };
    };
    quote! {
        {
            let __text = #text_expr;
            let __runs = #runs_expr;
            ::quoin::build_styled_text(__text, &__runs)
        }
    }
}
