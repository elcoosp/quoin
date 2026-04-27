use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node, handler::wrap_event_handler};

pub(crate) fn emit_radio_group(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let children: Vec<TokenStream> = el
        .children
        .iter()
        .map(|c| emit_node(c, bindings, inside_for))
        .collect();
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("RadioGroup_{}", next_extract_id());
            let comp_ident = quote::format_ident!("RadioGroup");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class=#user_class }
        };
        quote! { <#tag #class_prop> #(#children)* </#tag> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let cls = if user_class.is_empty() {
            "flex flex-col gap-2"
        } else {
            &user_class
        };
        quote! { <div class=#cls> #(#children)* </div> }
    }
}

pub(crate) fn emit_radio(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let name_expr = find_arg_expr(el, "name");
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr = find_arg_expr(el, "on_change");
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    let base = "h-4 w-4 rounded-full border border-input ring-offset-background accent-primary-500 cursor-pointer";
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("RadioGroupItem_{}", next_extract_id());
            let comp_ident = quote::format_ident!("RadioGroupItem");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let value_prop = match value_expr {
            Some(val) => quote! { value=#val },
            None => quote! {},
        };
        let checked_prop = match checked_expr {
            Some(val) => quote! { checked=#val },
            None => quote! {},
        };
        let on_change_prop = match on_change_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_checked_change=#wrapped }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class=#user_class }
        };
        quote! { <#tag #value_prop #checked_prop #on_change_prop #class_prop disabled=#disabled /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let type_prop = quote! { r#type="radio"# };
        let name_prop = match name_expr {
            Some(n) => quote! { name=#n },
            None => quote! {},
        };
        let value_prop = match value_expr {
            Some(v) => quote! { value=#v },
            None => quote! {},
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
                let bind_name = quote::format_ident!("__quoin_radio_bind_{}", bind_id);
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
        let tag_ident = proc_macro2::Ident::new("input", proc_macro2::Span::call_site());
        quote! { <#tag_ident class=#full_class #type_prop #name_prop #value_prop #checked_prop #on_input_prop #disabled_prop /> }
    }
}
