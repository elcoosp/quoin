use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_nodes;

pub(crate) fn emit_if(if_node: &IfNode) -> TokenStream {
    let inner = emit_if_inner(if_node);
    wrap_with_cfg(&if_node.attrs, inner)
}

fn emit_if_inner(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = if else_branch.len() == 1 {
            match &else_branch[0] {
                RenderNode::If(nested_if) => emit_if(nested_if),
                _ => emit_nodes(else_branch),
            }
        } else {
            emit_nodes(else_branch)
        };
        quote! { { if #cond { #then_branch } else { #else_tokens } } }
    } else {
        quote! { { if #cond { #then_branch } } }
    }
}

pub(crate) fn emit_for(for_node: &ForNode) -> TokenStream {
    let inner = emit_for_inner(for_node);
    wrap_with_cfg(&for_node.attrs, inner)
}

fn emit_for_inner(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes(&for_node.body);
    quote! {
        {
            ::gpui::div().children(
                #iterable.into_iter().map(|#pat| { #body }).collect::<Vec<_>>()
            )
        }
    }
}
