use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, handler::wrap_event_handler};

pub(crate) fn emit_checkbox(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("Checkbox_{}", next_extract_id());
            let comp_ident = quote::format_ident!("Checkbox");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let checked_prop = match checked_expr {
            Some(val) => quote! { checked={ #val.into() } },
            None => quote! {},
        };
        let on_change_prop = match on_change_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_checked_change={ #wrapped.into() } }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class={ #user_class.into() } }
        };
        quote! { <#tag #checked_prop #on_change_prop #class_prop disabled={ #disabled.into() } /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let base = "h-4 w-4 rounded border border-input ring-offset-background accent-primary-500 cursor-pointer";
        let full_class = if user_class.is_empty() {
            base.to_string()
        } else {
            format!("{} {}", base, user_class)
        };

        let checked_prop = match checked_expr {
            Some(val) => {
                quote! { prop:checked={leptos::prelude::Signal::derive(move || #val)} }
            }
            None => quote! {},
        };

        let on_input_prop = match on_change_expr {
            Some(handler) => {
                let handler = wrap_event_handler(handler);
                let bind_id = next_extract_id();
                let bind_name = quote::format_ident!("__quoin_cb_bind_{}", bind_id);
                bindings.push(quote! {
                    let #bind_name = {
                        let __handler = #handler;
                        move |ev: leptos::ev::Event| {
                            let checked = leptos::prelude::event_target_checked(&ev);
                            __handler(checked);
                        }
                    };
                });
                quote! { on:input=#bind_name }
            }
            None => quote! {},
        };

        let disabled_prop = if disabled {
            quote! { disabled=true }
        } else {
            quote! {}
        };
        let type_prop = quote! { r#type="checkbox"# };

        let mut attrs: Vec<TokenStream> = vec![
            quote! { class=#full_class },
            type_prop,
            checked_prop,
            on_input_prop,
            disabled_prop,
        ];

        let tag_ident = proc_macro2::Ident::new("input", proc_macro2::Span::call_site());
        quote! { <#tag_ident #(#attrs)* /> }
    }
}
