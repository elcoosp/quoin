use crate::emit::common::find_arg_string;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_render_inner;

pub(crate) fn emit_tooltip(el: &Element) -> TokenStream {
    let trigger_expr = &el.trigger_expr;
    let text = find_arg_string(el, "text").unwrap_or_default();

    if trigger_expr.is_none() {
        return quote! { span { title: #text } {#text} };
    }

    let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone().unwrap()));

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! {
            shadcn_dioxus::tooltip::Tooltip {
                shadcn_dioxus::tooltip::TooltipTrigger { #trigger_inner }
                shadcn_dioxus::tooltip::TooltipContent { #text }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! {
            {
                let mut __tip_open = dioxus::prelude::use_signal(|| false);
                dioxus::prelude::rsx! {
                    div {
                        class: "relative inline-block",
                        onmouseenter: move |_| __tip_open.set(true),
                        onmouseleave: move |_| __tip_open.set(false),
                        #trigger_inner,
                        if *__tip_open.read() {
                            div {
                                class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                #text
                            }
                        }
                    }
                }
            }
        }
    }
}
