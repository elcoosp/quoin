use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render_inner, handler::wrap_dioxus_handler};

pub(crate) fn emit_dropdown_menu(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { div { "dropdown: missing trigger" } },
        };

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;
                    let handler = wrap_dioxus_handler(on_click);
                    Some(quote! {
                        shadcn_dioxus::dropdown_menu::DropdownMenuItem {
                            onclick: {#handler},
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone()));

        quote! {
            shadcn_dioxus::dropdown_menu::DropdownMenu {
                shadcn_dioxus::dropdown_menu::DropdownMenuTrigger {
                    #trigger_inner
                }
                shadcn_dioxus::dropdown_menu::DropdownMenuContent {
                    #(#item_tokens)*
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { div { "dropdown: missing trigger" } },
        };

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;

                    let checked = find_arg_bool(e, "checked");

                    let handler = wrap_dioxus_handler(on_click);
                    let check_mark = if checked { "\u{2713} " } else { "" };
                    Some(quote! {
                        div {
                            class: "px-3 py-2 cursor-pointer text-white hover:bg-gray-600 flex items-center",
                            onclick: {
                                let __item_handler = #handler;
                                move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.stop_propagation();
                                    __open.set(false);
                                    __item_handler(ev);
                                }
                            },
                            #check_mark
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone()));

        quote! {
            {
                let mut __open = dioxus::prelude::use_signal(|| false);
                dioxus::prelude::rsx! {
                    div {
                        class: "relative inline-block",
                        onclick: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                            ev.stop_propagation();
                            __open.toggle();
                        },
                        #trigger_inner,
                        if *__open.read() {
                            div {
                                class: "absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg",
                                onclick: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.stop_propagation();
                                },
                                onmousedown: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.prevent_default();
                                },
                                #(#item_tokens)*
                            }
                        }
                    }
                }
            }
        }
    }
}
