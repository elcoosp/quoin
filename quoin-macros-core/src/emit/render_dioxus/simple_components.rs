use crate::emit::common::{find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render_inner, generic::emit_html_el};

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------
pub(crate) fn emit_badge(el: &Element) -> TokenStream {
    let color_expr = find_arg_expr(el, "color");
    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let class_str = if let Some(color) = color_expr {
            let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
            match bg_class {
                Some(cls) => format!(
                    "inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}",
                    cls
                ),
                None => {
                    "inline-flex items-center px-1.5 rounded text-xs font-medium text-white bg-gray-600"
                        .to_string()
                }
            }
        } else {
            "inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white"
                .to_string()
        };

        if children.is_empty() {
            quote! { shadcn_dioxus::badge::Badge { class: #class_str } }
        } else {
            quote! { shadcn_dioxus::badge::Badge { class: #class_str, #(#children)* } }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        match color_expr {
            Some(color) => {
                let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
                match bg_class {
                    Some(cls) => quote! {
                        span {
                            class: format!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}", #cls),
                            #(#children)*
                        }
                    },
                    None => quote! {
                        span {
                            class: "inline-flex items-center px-1.5 rounded text-xs font-medium text-white bg-gray-600",
                            #(#children)*
                        }
                    },
                }
            }
            None => quote! {
                span { class: "inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white", #(#children)* }
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Scroll area
// ---------------------------------------------------------------------------
pub(crate) fn emit_scroll_area(el: &Element) -> TokenStream {
    let direction = find_arg_string(el, "direction").unwrap_or_else(|| "vertical".to_string());

    let overflow_class = match direction.as_str() {
        "horizontal" => "overflow-x-auto",
        "both" => "overflow-auto",
        _ => "overflow-y-auto",
    };

    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: format!("{} {}", #s, #overflow_class), });
                } else {
                    attrs.push(quote! { class: format!("{} {}", #value, #overflow_class), });
                }
            }
            "direction" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        attrs.push(quote! { class: #overflow_class, });
    }
    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    quote! { div { #(#attrs)* #(#children)* } }
}

// ---------------------------------------------------------------------------
// Clipboard button
// ---------------------------------------------------------------------------
pub(crate) fn emit_clipboard_button(el: &Element) -> TokenStream {
    let copy_text = match find_arg_expr(el, "copy_text") {
        Some(ct) => ct,
        None => return emit_html_el(el, "button"),
    };

    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: #s, });
                } else {
                    attrs.push(quote! { class: {#value}, });
                }
            }
            "disabled" => attrs.push(quote! { disabled: #value, }),
            "copy_text" => {}
            _ => {}
        }
    }

    let mut children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    if children.is_empty() {
        children.push(quote! { "Copy" });
    }

    quote! {
        button {
            #(#attrs)*
            onclick: move |_| {
                quoin::clipboard_write_text(&(#copy_text).to_string());
            },
            #(#children)*
        }
    }
}

// ---------------------------------------------------------------------------
// StyledText
// ---------------------------------------------------------------------------
pub(crate) fn emit_styled_text(el: &Element) -> TokenStream {
    let text_expr = find_arg_expr(el, "text");
    let query_expr = find_arg_expr(el, "query");

    match (text_expr, query_expr) {
        (Some(text), None) => {
            quote! { span { #text } }
        }
        (Some(text), Some(query)) => {
            quote! {
                {
                    fn __quoin_styled_text(__text_val: String, __query_val: String) -> dioxus::prelude::Element {
                        let mut __parts: Vec<dioxus::prelude::Element> = Vec::new();
                        if !__query_val.is_empty() {
                            let mut __remaining: &str = &__text_val;
                            let __query_lower: String = __query_val.to_lowercase();
                            while let Some(__idx) = __remaining.to_lowercase().find(&__query_lower) {
                                if __idx > 0 {
                                    let __before: String = __remaining[..__idx].to_string();
                                    __parts.push(dioxus::prelude::rsx! { span { "{__before}" } });
                                }
                                let __match_str: String = __remaining[__idx..__idx + __query_val.len()].to_string();
                                __parts.push(dioxus::prelude::rsx! {
                                    span { class: "bg-yellow-200 text-black", "{__match_str}" }
                                });
                                __remaining = &__remaining[__idx + __query_val.len()..];
                            }
                        }
                        if __parts.is_empty() {
                            __parts.push(dioxus::prelude::rsx! { span { "{__text_val}" } });
                        }
                        dioxus::prelude::rsx! { span { {__parts.into_iter()} } }
                    }
                    __quoin_styled_text((#text).clone(), (#query).clone())
                }
            }
        }
        (None, _) => {
            quote! { span {} }
        }
    }
}

// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------
pub(crate) fn emit_icon(el: &Element) -> TokenStream {
    let name = find_arg_string(el, "icon_name");
    let size_class = find_arg_expr(el, "class");

    let class_str = match size_class {
        Some(c) => quote! { format!("{} w-4 h-4 inline-block", #c) },
        None => quote! { "w-4 h-4 inline-block" },
    };

    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

    match name {
        Some(n) => {
            if let Some(svg) = crate::transpile::icon_codegen::icon_svg_html(&n) {
                quote! { span { class: #class_str, #svg } }
            } else {
                quote! { span { class: #class_str, "\u{2753}" } }
            }
        }
        None => {
            if children.is_empty() {
                quote! { span { class: #class_str, "\u{2753}" } }
            } else {
                quote! { span { class: #class_str, #(#children)* } }
            }
        }
    }
}
