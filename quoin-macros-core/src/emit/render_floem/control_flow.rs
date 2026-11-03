//! `if`/`for` for Floem: the dynamic seams.
//!
//! These are the only places where the emitted tree type-erases (`into_any`)
//! — the same cost a hand-written Floem application pays at the same seam.
//! `for` with a `key:` argument lowers to the diffing `dyn_stack`; without
//! one it lowers to `dyn_container` (full rebuild per change) and emits a
//! diagnostic suggesting the key.

use crate::render_ast::{ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_node;

use super::handler::wrap_closure_with as wrap_closure;

/// `if[c] { A } else { B }` / `if[c] { A }` → `dyn_container`.
///
/// `dyn_container(update_view, child_fn)` re-runs `update_view` reactively
/// and rebuilds the child when the value changes. `child_fn` receives the
/// condition value and picks a branch; both branches are `into_any()`-ed
/// because their types differ — exactly one seam per `if`.
pub(crate) fn emit_if(node: &IfNode, ctx: &mut super::EmitterCtx) -> TokenStream {
    let cond = &node.condition;
    let then_branch = emit_branch(&node.then_branch, ctx);

    let else_branch = if let Some(else_nodes) = &node.else_branch {
        emit_branch(else_nodes, ctx)
    } else {
        quote! { ::floem::views::Empty::new().into_any() }
    };

    // Both closures are `Fn`: shadow clones live outside each of them.
    let cond_shadows = super::handler::capture_shadows(cond, ctx);
    let update_fn = super::handler::wrap_closure_with(&cond_shadows, quote! { move || (#cond) });

    let branch_shadows: Vec<TokenStream> = branch_captures(node)
        .iter()
        .map(|id| quote! { let #id = ::core::clone::Clone::clone(&#id); })
        .collect();
    let child_fn = super::handler::wrap_closure_with(
        &branch_shadows,
        quote! { move |__c| if __c {
            #then_branch
        } else {
            #else_branch
        } },
    );

    quote! {
        ::floem::views::dyn_container(#update_fn, #child_fn)
    }
}

/// The captures of both `if` branches (conditions excluded; the update
/// closure handles those).
fn branch_captures(node: &IfNode) -> Vec<proc_macro2::Ident> {
    let mut nodes: Vec<RenderNode> = Vec::new();
    nodes.extend(node.then_branch.iter().cloned());
    if let Some(else_nodes) = &node.else_branch {
        nodes.extend(else_nodes.iter().cloned());
    }
    let mut pats = std::collections::HashSet::new();
    crate::transpile::collect_node_captures(&nodes, &mut pats)
}

/// One branch: a single node stays itself; multi-node branches become a
/// column stack (the plan's allowed structural normalization).
fn emit_branch(nodes: &[RenderNode], ctx: &mut super::EmitterCtx) -> TokenStream {
    let views: Vec<TokenStream> = nodes.iter().map(|n| emit_node(n, ctx, false)).collect();
    if views.len() == 1 {
        let only = &views[0];
        quote! { #only.into_any() }
    } else {
        let stack = super::generic::static_stack(&views, quoin_style::FlexDir::Column, None);
        quote! { #stack.into_any() }
    }
}

/// `for[item in list]` → `dyn_stack` (keyed) or `dyn_container` (unkeyed).
pub(crate) fn emit_for(node: &ForNode, ctx: &mut super::EmitterCtx) -> TokenStream {
    let pat = &node.pat;
    let iterable = &node.iterable;

    // The loop pattern introduces a local; the hoist must not shadow it.
    ctx.excluded.insert(node.pat.to_string());

    let body = {
        let views: Vec<TokenStream> = node.body.iter().map(|n| emit_node(n, ctx, true)).collect();
        match views.len() {
            1 => views[0].clone(),
            _ => super::generic::static_stack(&views, quoin_style::FlexDir::Column, None),
        }
    };

    // `Fn` closures cannot capture by value, so every closure site gets its
    // own shadow clones in an enclosing block (the clones read the hoisted
    // handles; the closures own their copies).
    let iterable_shadows = super::handler::capture_shadows(iterable, ctx);
    let key_shadows = node
        .key
        .as_ref()
        .map(|key| super::handler::capture_shadows(key, ctx))
        .unwrap_or_default();
    let mut body_pats = std::collections::HashSet::new();
    body_pats.insert(node.pat.to_string());
    let body_captures = crate::transpile::collect_node_captures(&node.body, &mut body_pats);
    let body_shadows: Vec<TokenStream> = body_captures
        .iter()
        .map(|id| quote! { let #id = ::core::clone::Clone::clone(&#id); })
        .collect();

    if let Some(key_expr) = &node.key {
        // Keyed: dyn_stack diffs by the key expression. `key_fn` sees
        // `&T`, so the item is cloned for the key scope; the view
        // closure owns the item by value.
        let each_fn = wrap_closure(&iterable_shadows, quote! { move || (#iterable) });
        let key_fn = wrap_closure(
            &key_shadows,
            quote! { move |__item| {
                let #pat = ::core::clone::Clone::clone(__item);
                let _ = &#pat;
                #key_expr
            } },
        );
        let view_fn = wrap_closure(&body_shadows, quote! { move |#pat| #body });
        quote! {
            ::floem::views::dyn_stack(#each_fn, #key_fn, #view_fn)
        }
    } else {
        // Unkeyed: rebuild the whole list on change — the same cost a
        // hand-written `dyn_container` pays — plus a diagnostic that
        // points at the missing key.
        let diagnostic = quote! {
            ::core::compile_error!(
                "unkeyed `for[...]` on the Floem backend rebuilds every row on each change; \
                 add a key (`for[item in list, key: item.id]`) to get diffing"
            );
        };
        quote! {
            {
                #diagnostic
                ::floem::views::dyn_container(
                    move || {
                        #(#iterable_shadows)*
                        ::std::sync::Arc::new(
                            (#iterable).into_iter().collect::<Vec<_>>()
                        )
                    },
                    move |__items| {
                        let __kids = __items
                            .iter()
                            .map(|#pat| ::core::clone::Clone::clone(#pat))
                            .map(|#pat| #body)
                            .collect::<Vec<_>>();
                        ::floem::views::Stack::vertical_from_iter(__kids).into_any()
                    },
                )
            }
        }
    }
}
