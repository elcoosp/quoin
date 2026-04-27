use crate::emit::common::find_arg_expr;
use crate::render_ast::{Element, RenderNode};
use crate::transpile::{collect_handler_idents, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn emit_tabs(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let on_click_with_move = force_move_on_closure(on_click_expr);

        let tab_triggers: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;
                    let index_clone = index.clone();
                    Some(quote! {
                        shadcn_dioxus::tabs::TabsTrigger {
                            value: "{#index.to_string()}",
                            onclick: {
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#index_clone); }
                            },
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            shadcn_dioxus::tabs::Tabs {
                value: "{#active_expr.to_string()}",
                shadcn_dioxus::tabs::TabsList {
                    #(#tab_triggers)*
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let param_idents: Vec<proc_macro2::Ident> =
            if let syn::Expr::Closure(closure) = on_click_expr {
                closure
                    .inputs
                    .iter()
                    .filter_map(|pat| {
                        if let syn::Pat::Ident(pat_ident) = pat {
                            Some(pat_ident.ident.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

        let param_names: std::collections::HashSet<String> =
            param_idents.iter().map(|id| id.to_string()).collect();

        let body_idents: Vec<proc_macro2::Ident> = collect_handler_idents(on_click_expr)
            .into_iter()
            .filter(|id| !param_names.contains(&id.to_string()))
            .collect();

        let on_click_with_move = force_move_on_closure(on_click_expr);

        let tab_elements: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;

                    let param_shadows: Vec<TokenStream> = param_idents
                        .iter()
                        .map(|id| quote! { let #id = #index; })
                        .collect();
                    let clone_shadows: Vec<TokenStream> = body_idents
                        .iter()
                        .map(|id| quote! { let #id = #id.clone(); })
                        .collect();
                    let call_args: Vec<TokenStream> =
                        param_idents.iter().map(|id| quote! { #id }).collect();

                    Some(quote! {
                        div {
                            class: if #index == #active_expr {
                                "px-4 py-2 cursor-pointer text-white"
                            } else {
                                "px-4 py-2 cursor-pointer text-gray-400"
                            },
                            onclick: {
                                #(#param_shadows)*
                                #(#clone_shadows)*
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#(#call_args)*) }
                            },
                            {#label}
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            div { class: "flex", #(#tab_elements)* }
        }
    }
}
