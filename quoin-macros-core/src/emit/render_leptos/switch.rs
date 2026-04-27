use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, handler::wrap_event_handler};

pub(crate) fn emit_switch(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("Switch_{}", next_extract_id());
            let comp_ident = quote::format_ident!("Switch");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let checked_prop = match checked_expr {
            Some(val) => quote! { checked={#val} },
            None => quote! {},
        };
        let on_change_prop = match on_change_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_checked_change={#wrapped} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class={#user_class} }
        };
        quote! { <#tag #checked_prop #on_change_prop #class_prop disabled={#disabled} /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let track_cls = "relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors";
        let track_on_cls = "bg-primary";
        let track_off_cls = "bg-input";
        let thumb_cls = "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform";
        let thumb_on_cls = "translate-x-5";
        let thumb_off_cls = "translate-x-0";
        let disabled_cls = if disabled {
            " opacity-50 pointer-events-none"
        } else {
            ""
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
                let bind_name = quote::format_ident!("__quoin_sw_bind_{}", bind_id);
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

        let role_prop = quote! { role="switch" };
        let type_prop = quote! { r#type="checkbox"# };

        let track_class_id = next_extract_id();
        let track_class_name = quote::format_ident!("__quoin_sw_track_{}", track_class_id);
        let thumb_class_id = next_extract_id();
        let thumb_class_name = quote::format_ident!("__quoin_sw_thumb_{}", thumb_class_id);
        bindings.push(quote! {
            let #track_class_name = leptos::prelude::Signal::derive(move || {
                if #checked_expr.map(|v| v.clone()).unwrap_or(false) {
                    concat!(#track_on_cls, #disabled_cls)
                } else {
                    concat!(#track_off_cls, #disabled_cls)
                }
            });
            let #thumb_class_name = leptos::prelude::Signal::derive(move || {
                if #checked_expr.map(|v| v.clone()).unwrap_or(false) {
                    concat!(#thumb_cls, " ", #thumb_on_cls)
                } else {
                    concat!(#thumb_cls, " ", #thumb_off_cls)
                }
            });
        });

        let full_cls = if user_class.is_empty() {
            track_cls.to_string()
        } else {
            format!("{} {}", track_cls, user_class)
        };

        quote! {
            <button
                type="button"
                role="switch"
                class=#full_cls
                disabled={#disabled}
                on:click=move |_| {}
            >
                <input #type_prop #checked_prop #on_input_prop class="sr-only peer" />
                <div class={#track_class_name} />
                <div class={#thumb_class_name} />
            </button>
        }
    }
}
