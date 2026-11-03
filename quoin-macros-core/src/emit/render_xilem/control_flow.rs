//! `if`/`for` for the Xilem backend: the dynamic seams.
//!
//! - `if[c] { A } else { B }` → `if c { A.boxed() } else { B.boxed() }`
//!   (the plan's specified seam; both branches type-erase into
//!   `AnyWidgetView`).
//! - `if[c] { A }` without else → `(c).then(|| A)` as an `Option` child —
//!   no boxing at all (`Option<V>` is a view sequence).
//! - `for[item in list]` →
//!   `{ item-shadows; list.into_iter().map(move |item| body).collect() }` —
//!   the `Vec<V>` view sequence diffs by index; the shadow block gives the
//!   map closure its own copies so several closures can share one signal.

use crate::render_ast::{ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, handler};

pub(crate) fn emit_if(node: &IfNode, ctx: &mut super::EmitterCtx) -> TokenStream {
    let cond = &node.condition;
    let then_branch = emit_branch(&node.then_branch, ctx);

    match &node.else_branch {
        Some(else_nodes) => {
            let else_branch = emit_branch(else_nodes, ctx);
            // One seam: both branches type-erase into AnyWidgetView.
            quote! {
                if #cond {
                    #then_branch.boxed()
                } else {
                    #else_branch.boxed()
                }
            }
        }
        None => {
            // `Option<V>` child: no boxing.
            quote! {
                (#cond).then(|| #then_branch)
            }
        }
    }
}

fn emit_branch(nodes: &[RenderNode], ctx: &mut super::EmitterCtx) -> TokenStream {
    let views: Vec<TokenStream> = nodes.iter().map(|n| emit_node(n, ctx, false)).collect();
    if views.len() == 1 {
        views[0].clone()
    } else {
        super::generic::flex_sequence(&views, quoin_style::FlexDir::Column)
    }
}

pub(crate) fn emit_for(node: &ForNode, ctx: &mut super::EmitterCtx) -> TokenStream {
    let pat = &node.pat;
    let iterable = &node.iterable;

    // The loop pattern introduces a local; the hoist must not shadow it.
    ctx.excluded.insert(pat.to_string());

    let body = {
        let views: Vec<TokenStream> = node.body.iter().map(|n| emit_node(n, ctx, true)).collect();
        match views.len() {
            1 => views[0].clone(),
            _ => super::generic::flex_sequence(&views, quoin_style::FlexDir::Column),
        }
    };

    // The map closure owns its captures: shadow-clones live outside it, so
    // several closures in the same rebuild pass can share one signal.
    let mut body_pats = std::collections::HashSet::new();
    body_pats.insert(pat.to_string());
    let body_captures = crate::transpile::collect_node_captures(&node.body, &mut body_pats);
    let body_shadows: Vec<TokenStream> = body_captures
        .iter()
        .map(|id| {
            ctx.capture(id);
            quote! { let #id = ::core::clone::Clone::clone(&#id); }
        })
        .collect();

    handler::wrap_closure_with(
        &body_shadows,
        quote! {
            (#iterable).into_iter().map(move |#pat| #body).collect::<::std::vec::Vec<_>>()
        },
    )
}
