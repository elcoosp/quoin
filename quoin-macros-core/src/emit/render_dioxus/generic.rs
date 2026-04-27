use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render_inner, handler::wrap_dioxus_handler};

pub(crate) fn emit_html_el(el: &Element, name_str: &str) -> TokenStream {
    emit_html_el_inner(el, name_str)
}

pub(crate) fn emit_html_el_inner(el: &Element, name_str: &str) -> TokenStream {
    let tag = match name_str {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        "button" => "button",
        "input" => "input",
        "span" => "span",
        "label" => "label",
        "a" => "a",
        "ul" => "ul",
        "ol" => "ol",
        "li" => "li",
        "hr" => "hr",
        "br" => "br",
        "textarea" => "textarea",
        "select" => "select",
        "form" => "form",
        "img" => "img",
        _ => "div",
    };

    let has_value = el.args.iter().any(|a| a.key == "value");
    let has_on_input = el.args.iter().any(|a| a.key == "on_input");
    let auto_bind_input = tag == "input" && has_value && !has_on_input;

    let mut attrs: Vec<TokenStream> = Vec::new();
    let mut children: Vec<TokenStream> = Vec::new();
    let mut children_attr_expr: Option<&syn::Expr> = None;

    for arg in &el.args {
        let key_str = arg.key.to_string();
        let key_ident = arg.key.clone();
        let value = &arg.value;
        match key_str.as_str() {
            "children" => {
                children_attr_expr = Some(value);
            }
            "on_click" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onclick: {#handler}, })
            }
            "on_mouse_down" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmousedown: {#handler}, })
            }
            "on_mouse_up" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseup: {#handler}, })
            }
            "on_mouse_enter" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseenter: {#handler}, })
            }
            "on_mouse_leave" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseleave: {#handler}, })
            }
            "on_input" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { oninput: {#handler}, })
            }
            "on_change" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onchange: {#handler}, })
            }
            "value" => {
                if tag == "input" {
                    attrs.push(quote! { value: {#value.get()}, });
                } else {
                    attrs.push(quote! { value: {#value}, });
                }
            }
            "primary" | "ghost" | "destructive" | "active" | "trigger" | "rows" | "striped"
            | "items" | "estimated_height" | "copy_text" | "sortable" | "width" | "resizable"
            | "selectable" | "on_sort" | "bordered" | "size" | "navigate_to" | "cfg" | "label"
            | "render" | "key" | "index" | "text" | "query" | "color" | "direction" | "tooltip"
            | "icon_name" => {}
            _ => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { #key_ident: #s, });
                } else {
                    attrs.push(quote! { #key_ident: {#value}, });
                }
            }
        }
    }

    if auto_bind_input {
        let value_expr = find_arg_expr(el, "value").unwrap();
        attrs.push(quote! {
            oninput: move |ev: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                #value_expr.set(ev.value());
            },
        });
    }

    if let Some(expr) = children_attr_expr {
        children.push(quote! { { #expr.into_iter() } });
    }

    if let Some(children_expr) = &el.children_expr {
        children.push(quote! { { #children_expr.into_iter() } });
    }

    for child in &el.children {
        children.push(emit_render_inner(child));
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());

    if attrs.is_empty() && children.is_empty() {
        quote! { #tag_ident {} }
    } else if attrs.is_empty() {
        quote! { #tag_ident { #(#children)* } }
    } else if children.is_empty() {
        quote! { #tag_ident { #(#attrs)* } }
    } else {
        quote! { #tag_ident { #(#attrs)* #(#children)* } }
    }
}
