use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{generic::emit_html_el_inner, handler::wrap_dioxus_handler};

pub(crate) fn emit_input(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();

        let class_expr = find_arg_expr(el, "class");
        let value_expr = find_arg_expr(el, "value");
        let on_input_expr = find_arg_expr(el, "on_input");
        let disabled = find_arg_bool(el, "disabled");

        let base_class = "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

        let full_class = match class_expr {
            Some(cls) => quote! { format!("{} {}", #base_class, #cls) },
            None => quote! { #base_class },
        };

        let placeholder_attr = if placeholder.is_empty() {
            quote! {}
        } else {
            quote! { placeholder: #placeholder, }
        };

        let disabled_attr = if disabled {
            quote! { disabled: true, }
        } else {
            quote! {}
        };

        let value_attr = if let Some(val) = value_expr {
            quote! { value: {#val.get()}, }
        } else {
            quote! {}
        };

        let oninput_attr = if let Some(handler) = on_input_expr {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { oninput: #wrapped, }
        } else if let Some(val) = value_expr {
            quote! {
                oninput: move |ev: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                    #val.set(ev.value());
                },
            }
        } else {
            quote! {}
        };

        quote! {
            input {
                class: #full_class,
                #placeholder_attr
                #value_attr
                #oninput_attr
                #disabled_attr
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_html_el_inner(el, "input")
    }
}
