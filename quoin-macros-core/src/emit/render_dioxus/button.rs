use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render_inner, generic::emit_html_el_inner, handler::wrap_dioxus_handler};

pub(crate) fn emit_button(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");
        let class_expr = find_arg_expr(el, "class");

        let primary = find_arg_bool(el, "primary");
        let destructive = find_arg_bool(el, "destructive");
        let ghost = find_arg_bool(el, "ghost");
        let disabled = find_arg_bool(el, "disabled");

        let variant_class = if destructive {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        } else if ghost {
            "hover:bg-accent hover:text-accent-foreground"
        } else if primary {
            "bg-primary text-primary-foreground hover:bg-primary/90"
        } else {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        };

        let base_class = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors cursor-pointer";
        let disabled_class = if disabled {
            " opacity-50 pointer-events-none"
        } else {
            ""
        };

        let full_class = match class_expr {
            Some(cls) => {
                quote! { format!("{} {}{} {}", #base_class, #variant_class, #disabled_class, #cls) }
            }
            None => quote! { format!("{} {}{}", #base_class, #variant_class, #disabled_class) },
        };

        let on_click_attr = find_arg_expr(el, "on_click").map(|handler_expr| {
            let handler = wrap_dioxus_handler(handler_expr);
            quote! { onclick: {#handler}, }
        });

        let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

        let inner_button = if children.is_empty() {
            quote! {
                button { class: #full_class, #on_click_attr }
            }
        } else {
            quote! {
                button { class: #full_class, #on_click_attr #(#children)* }
            }
        };

        match tooltip_text {
            Some(text) => quote! {
                {
                    let mut __tip_open = dioxus::prelude::use_signal(|| false);
                    dioxus::prelude::rsx! {
                        div {
                            class: "relative inline-flex",
                            onmouseenter: move |_| __tip_open.set(true),
                            onmouseleave: move |_| __tip_open.set(false),
                            #inner_button,
                            if *__tip_open.read() {
                                div {
                                    class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                    #text
                                }
                            }
                        }
                    }
                }
            },
            None => inner_button,
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let inner_button = emit_html_el_inner(el, "button");

        match tooltip_text {
            Some(text) => quote! {
                {
                    let mut __tip_open = dioxus::prelude::use_signal(|| false);
                    dioxus::prelude::rsx! {
                        div {
                            class: "relative inline-flex",
                            onmouseenter: move |_| __tip_open.set(true),
                            onmouseleave: move |_| __tip_open.set(false),
                            #inner_button,
                            if *__tip_open.read() {
                                div {
                                    class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                    #text
                                }
                            }
                        }
                    }
                }
            },
            None => inner_button,
        }
    }
}
