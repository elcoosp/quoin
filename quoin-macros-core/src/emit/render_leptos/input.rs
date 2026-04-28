use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, generic::emit_html_tag_inner, handler::wrap_event_handler};

pub(crate) fn emit_input(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();

        let class_expr = find_arg_expr(el, "class");
        let value_expr = find_arg_expr(el, "value");
        let on_change_expr = find_arg_expr(el, "on_change");
        let on_input_expr = find_arg_expr(el, "on_input");
        let disabled = find_arg_bool(el, "disabled");

        let has_explicit_handler = on_change_expr.is_some() || on_input_expr.is_some();
        let needs_auto_bind = value_expr.is_some() && !has_explicit_handler;

        let value_prop: TokenStream = if let Some(val) = value_expr {
            quote! {
                value={
                    let __val = (#val).clone();
                    leptos::prelude::Signal::derive(move || __val.get())
                }
            }
        } else {
            quote! {}
        };

        let on_change_prop: TokenStream = if let Some(handler) = on_change_expr {
            let wrapped = wrap_event_handler(handler);
            quote! { on_change={ #wrapped.into() } }
        } else if let Some(handler) = on_input_expr {
            let wrapped = wrap_event_handler(handler);
            quote! { on_change={ #wrapped.into() } }
        } else if needs_auto_bind {
            let val = value_expr.unwrap();
            let bind_id = next_extract_id();
            let bind_name = quote::format_ident!("__quoin_input_bind_{}", bind_id);
            bindings.push(quote! {
                let #bind_name = {
                    let __signal = (#val).clone();
                    move |val: String| {
                        __signal.set(val);
                    }
                };
            });
            quote! { on_change=#bind_name }
        } else {
            quote! {}
        };

        let placeholder_prop: TokenStream = if placeholder.is_empty() {
            quote! {}
        } else {
            quote! { placeholder={ #placeholder.into() } }
        };

        let class_prop: TokenStream = if let Some(cls) = class_expr {
            quote! { class={ #cls.into() } }
        } else {
            quote! {}
        };

        let disabled_prop: TokenStream = if disabled {
            quote! { disabled=true }
        } else {
            quote! {}
        };

        let input_alias = quote::format_ident!("Input_{}", next_extract_id());
        bindings.push(quote! {
            let #input_alias = leptos_shadcn_ui::Input;
        });

        quote! { <#input_alias #value_prop #on_change_prop #placeholder_prop #class_prop #disabled_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        emit_html_tag_inner(el, "input", bindings, inside_for)
    }
}
