use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node, handler::wrap_event_handler};

pub(crate) fn emit_html_tag(
    el: &Element,
    tag: &str,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    emit_html_tag_inner(el, tag, bindings, inside_for)
}

pub(crate) fn emit_html_tag_inner(
    el: &Element,
    tag: &str,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let mut attrs = Vec::new();

    let has_value = el.args.iter().any(|a| a.key == "value");
    let has_on_input = el.args.iter().any(|a| a.key == "on_input");
    let auto_bind_input = tag == "input" && has_value && !has_on_input;

    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "placeholder" => attrs.push(quote! { placeholder=#value }),
            "disabled" => attrs.push(quote! { disabled=#value }),
            "on_click" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:click=#handler })
            }
            "on_mouse_down" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:mousedown=#handler })
            }
            "on_input" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:input=#handler })
            }
            "on_change" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:change=#handler })
            }
            "value" => {
                if tag == "input" {
                    attrs.push(quote! { prop:value={
                        let __val = (#value).clone();
                        move || __val.get()
                    }});
                } else {
                    attrs.push(quote! { value={#value} });
                }
            }
            _ => {}
        }
    }

    if auto_bind_input {
        let value_expr = find_arg_expr(el, "value").unwrap();
        let bind_id = next_extract_id();
        let bind_name = quote::format_ident!("__quoin_input_bind_{}", bind_id);
        bindings.push(quote! {
            let #bind_name = {
                let __signal = (#value_expr).clone();
                move |ev: leptos::ev::Event| {
                    __signal.set(leptos::prelude::event_target_value(&ev));
                }
            };
        });
        attrs.push(quote! { on:input=#bind_name });
    }

    let mut children = Vec::new();
    if let Some(children_expr) = &el.children_expr {
        children.push(quote! {
            {#children_expr.into_iter().map(|v| v.into_any()).collect::<Vec<_>>()}
        });
    } else {
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    let is_void = matches!(tag, "input" | "hr" | "br" | "img");
    if is_void {
        quote! { <#tag_ident #(#attrs)* /> }
    } else if children.is_empty() {
        quote! { <#tag_ident #(#attrs)*></#tag_ident> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children)* </#tag_ident> }
    }
}
