use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_node;

// ---------------------------------------------------------------------------
// If nodes
// ---------------------------------------------------------------------------

pub(crate) fn emit_if(if_node: &IfNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = emit_if_reactive(if_node, bindings, inside_for);
    // Wrap in a block to prevent the `move ||` tokens from being parsed as
    // literal children by leptos::view!.
    let guarded = quote! { { #inner } };
    wrap_with_cfg(&if_node.attrs, guarded)
}

fn emit_if_reactive(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let body = build_if_body(if_node, bindings, inside_for);
    quote! {
        move || { use leptos::prelude::*; #body }
    }
}

pub(crate) fn build_if_body(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let cond_expr = &if_node.condition;

    let then_tokens: Vec<TokenStream> = if_node
        .then_branch
        .iter()
        .map(|n| emit_node(n, bindings, inside_for))
        .collect();
    let then_view = quote! { #(#then_tokens)* };

    match &if_node.else_branch {
        Some(else_branch) => {
            if else_branch.len() == 1 {
                if let RenderNode::If(nested_if) = &else_branch[0] {
                    let nested_body = build_if_body(nested_if, bindings, inside_for);
                    return quote! {
                        if #cond_expr {
                            { leptos::view! { #then_view } }.into_any()
                        } else {
                            #nested_body
                        }
                    };
                }
            }
            let else_tokens: Vec<TokenStream> = else_branch
                .iter()
                .map(|n| emit_node(n, bindings, inside_for))
                .collect();
            let else_view = quote! { #(#else_tokens)* };
            quote! {
                if #cond_expr {
                    { leptos::view! { #then_view } }.into_any()
                } else {
                    { leptos::view! { #else_view } }.into_any()
                }
            }
        }
        None => {
            quote! {
                (#cond_expr).then(|| { leptos::view! { #then_view } }.into_any())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// For nodes
// ---------------------------------------------------------------------------

pub(crate) fn emit_for(for_node: &ForNode, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let inner = emit_for_inner(for_node, bindings);
    let guarded = quote! { { #inner } };
    wrap_with_cfg(&for_node.attrs, guarded)
}

fn emit_for_inner(for_node: &ForNode, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body_tokens: Vec<TokenStream> = for_node
        .body
        .iter()
        .map(|n| emit_node(n, bindings, true))
        .collect();
    let body_view = quote! { #(#body_tokens)* };

    let iter_id = super::bindings::next_extract_id();
    let iter_name = quote::format_ident!("__quoin_for_iter_{}", iter_id);
    bindings.push(quote! { let #iter_name = (#iterable).clone(); });

    // Each map iteration produces a leptos::Fragment by wrapping
    // the view inside a move closure. This prevents any raw token
    // leakage (like "move ||") into the DOM.
    quote! {
        {
            #iter_name.clone().into_iter().map(|#pat| {
                { leptos::view! { #body_view } }.into_any()
            }).collect::<Vec<_>>()
        }
    }
}
