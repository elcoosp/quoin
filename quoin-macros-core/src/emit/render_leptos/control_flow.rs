use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{ForNode, IfNode, RenderNode};
use crate::transpile::collect_render_idents;
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_node;

pub(crate) fn emit_if(if_node: &IfNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = emit_if_reactive(if_node, bindings, inside_for);
    let guarded = quote! { { #inner } };
    wrap_with_cfg(&if_node.attrs, guarded)
}

fn emit_if_reactive(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    // Collect all idents used in then/else branches BEFORE building the body,
    // so the signal clones are available to child emitters.
    let mut idents = collect_render_idents(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        idents.extend(collect_render_idents(else_branch));
    }
    idents.sort_by_key(|id| id.to_string());
    idents.dedup_by(|a, b| *a == *b);

    // Push clones into bindings — these become `let x = x.clone();` statements
    // in the outer render closure, making it FnMut.
    for id in &idents {
        bindings.push(quote! { let #id = #id.clone(); });
    }

    // Now build the body; child emitters may also push to bindings.
    let body = build_if_body(if_node, bindings, inside_for);

    // Add a second layer of clones inside the move || closure itself.
    // This ensures each invocation of the inner closure gets its own
    // fresh clone of any captured signal, preventing moves that would
    // consume the outer binding.
    let inner_shadows: Vec<_> = idents.iter()
        .map(|id| quote! { let #id = #id.clone(); })
        .collect();

    quote! {
        {
            #(#inner_shadows)*
            move || { use leptos::prelude::*; #body }
        }
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

pub(crate) fn emit_for(for_node: &ForNode, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let inner = emit_for_inner(for_node, bindings);
    let guarded = quote! { { #inner } };
    wrap_with_cfg(&for_node.attrs, guarded)
}

/// Recursively collect all ident names bound by a pattern.
fn collect_pat_idents(pat: &syn::Pat) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    collect_pat_idents_inner(pat, &mut set);
    set
}

fn collect_pat_idents_inner(pat: &syn::Pat, out: &mut std::collections::HashSet<String>) {
    match pat {
        syn::Pat::Ident(pi) => { out.insert(pi.ident.to_string()); }
        syn::Pat::Tuple(t) => t.elems.iter().for_each(|p| collect_pat_idents_inner(p, out)),
        syn::Pat::TupleStruct(ts) => ts.elems.iter().for_each(|p| collect_pat_idents_inner(p, out)),
        syn::Pat::Struct(s) => s.fields.iter().for_each(|f| collect_pat_idents_inner(&f.pat, out)),
        syn::Pat::Reference(r) => collect_pat_idents_inner(&r.pat, out),
        syn::Pat::Type(t) => collect_pat_idents_inner(&t.pat, out),
        syn::Pat::Or(o) => o.cases.iter().for_each(|p| collect_pat_idents_inner(p, out)),
        syn::Pat::Slice(s) => s.elems.iter().for_each(|p| collect_pat_idents_inner(p, out)),
        _ => {}
    }
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

    // Collect idents referenced in the loop body, but exclude those bound by the loop pattern.
    let pat_str = pat.to_string();
    let mut body_idents = collect_render_idents(&for_node.body);
    body_idents.retain(|id| id.to_string() != pat_str);

    for id in &body_idents {
        bindings.push(quote! { let #id = #id.clone(); });
    }

    let iter_id = super::bindings::next_extract_id();
    let iter_name = quote::format_ident!("__quoin_for_iter_{}", iter_id);
    bindings.push(quote! { let #iter_name = (#iterable).clone(); });

    quote! {
        #iter_name.clone().into_iter().map(|#pat| {
            { leptos::view! { #body_view } }.into_any()
        }).collect::<Vec<_>>()
    }
}
