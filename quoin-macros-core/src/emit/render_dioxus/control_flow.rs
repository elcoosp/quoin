use crate::render_ast::{ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_nodes_inner;

pub(crate) fn emit_if(if_node: &IfNode) -> TokenStream {
    emit_if_inner(if_node)
}

fn emit_if_inner(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens = emit_nodes_inner(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        if else_branch.len() == 1 {
            if let RenderNode::If(nested_if) = &else_branch[0] {
                let nested = emit_if_inner(nested_if);
                return quote! { if #cond { #then_tokens } else #nested };
            }
        }
        let else_tokens = emit_nodes_inner(else_branch);
        quote! { if #cond { #then_tokens } else { #else_tokens } }
    } else {
        quote! { if #cond { #then_tokens } }
    }
}

pub(crate) fn emit_for(for_node: &ForNode) -> TokenStream {
    emit_for_inner(for_node)
}

fn emit_for_inner(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes_inner(&for_node.body);
    quote! { for #pat in #iterable { #body } }
}
