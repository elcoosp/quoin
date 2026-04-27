use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, handler::wrap_event_handler};

pub(crate) fn emit_slider(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let min_expr = find_arg_expr(el, "min");
    let max_expr = find_arg_expr(el, "max");
    let step_expr = find_arg_expr(el, "step");
    let on_input_expr = find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"));
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("Slider_{}", next_extract_id());
            let comp_ident = quote::format_ident!("Slider");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let value_prop = match value_expr {
            Some(val) => quote! { value={#val} },
            None => quote! {},
        };
        let min_prop = match min_expr {
            Some(m) => quote! { min={#m} },
            None => quote! {},
        };
        let max_prop = match max_expr {
            Some(m) => quote! { max={#m} },
            None => quote! {},
        };
        let step_prop = match step_expr {
            Some(s) => quote! { step={#s} },
            None => quote! {},
        };
        let on_change_prop = match on_input_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_value_change={#wrapped} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class={#user_class} }
        };
        quote! { <#tag #value_prop #min_prop #max_prop #step_prop #on_change_prop #class_prop disabled={#disabled} /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let base = "w-full h-2 rounded-lg appearance-none cursor-pointer accent-primary-500 bg-transparent";
        let full_class = if user_class.is_empty() {
            base.to_string()
        } else {
            format!("{} {}", base, user_class)
        };

        let type_prop = quote! { r#type="range"# };
        let min_prop = match min_expr {
            Some(m) => quote! { min={#m} },
            None => quote! {},
        };
        let max_prop = match max_expr {
            Some(m) => quote! { max={#m} },
            None => quote! {},
        };
        let step_prop = match step_expr {
            Some(s) => quote! { step={#s} },
            None => quote! {},
        };

        let value_prop = match value_expr {
            Some(val) => {
                quote! { prop:value={leptos::prelude::Signal::derive(move || #val)} }
            }
            None => quote! {},
        };

        let on_input_prop = match on_input_expr {
            Some(handler) => {
                let handler = wrap_event_handler(handler);
                let bind_id = next_extract_id();
                let bind_name = quote::format_ident!("__quoin_slider_bind_{}", bind_id);
                bindings.push(quote! {
                    let #bind_name = {
                        let __handler = #handler;
                        move |ev: leptos::ev::Event| {
                            let val = leptos::prelude::event_target_value(&ev);
                            __handler(val);
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
        quote! { <#tag_ident class=#full_class #type_prop #value_prop #min_prop #max_prop #step_prop #on_input_prop #disabled_prop /> }
    }
}
