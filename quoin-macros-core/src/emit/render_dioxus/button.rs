// Imports in this module vary with the `leptos-shadcn`/`dioxus-shadcn` feature:
// some are only used when the shadcn emit path is active. A file-level allow
// keeps every feature combination compiling without per-line noise.

#[cfg(feature = "dioxus-shadcn")]
use super::handler::wrap_dioxus_handler;
use crate::emit::common::find_arg_string;
#[cfg(feature = "dioxus-shadcn")]
use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

#[cfg(feature = "dioxus-shadcn")]
use super::emit_render_inner;
#[cfg(not(feature = "dioxus-shadcn"))]
use super::generic::emit_html_el_inner;

pub(crate) fn emit_button(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");
        let class_expr = find_arg_expr(el, "class");
        let disabled = find_arg_bool(el, "disabled");

        // Canonical `variant:` first (`emit::shadcn_spec`), boolean sugar
        // as the fallback; unknown values are a hard error. The variant
        // lowers to its shadcn Tailwind class string (the Dioxus shadcn
        // path is class-based — no component import needed).
        let choice = crate::emit::shadcn_spec::choice::canonical_button_variant(el);
        if let Some(error) = choice
            .as_ref()
            .and_then(crate::emit::shadcn_spec::choice::choice_error)
        {
            return error;
        }
        let variant_class = match choice.and_then(std::result::Result::ok) {
            Some("destructive") => {
                "bg-destructive text-destructive-foreground hover:bg-destructive/90"
            }
            Some("ghost") => "hover:bg-accent hover:text-accent-foreground",
            Some("secondary") => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            Some("link") => "text-primary underline-offset-4 hover:underline",
            Some("outline") | None => {
                "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
            }
            // `default`/`primary` — the emphasized filled look.
            Some(_) => "bg-primary text-primary-foreground hover:bg-primary/90",
        };

        let base_class = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors cursor-pointer";
        let disabled_class = if disabled {
            " opacity-50 pointer-events-none"
        } else {
            ""
        };

        let full_class = if let Some(cls) = class_expr {
            quote! { format!("{} {}{} {}", #base_class, #variant_class, #disabled_class, #cls) }
        } else {
            quote! { format!("{} {}{}", #base_class, #variant_class, #disabled_class) }
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
