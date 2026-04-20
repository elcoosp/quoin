use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => {
            let text = t.value();
            quote! { #text }
        }
        RenderNode::Expr(e) => {
            quote! { {#e} }
        }
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::ForEach(fe) => emit_for_each(fe),
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let tag = match name_str.as_str() {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        "button" => "button",
        "input" => "input",
        _ => "div",
    };

    // Build attribute pairs
    let mut attr_tokens = Vec::new();
    for (key, value) in &el.args {
        let key_str = key.to_string();
        let key_ident = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
        attr_tokens.push(quote! { #key_ident: #value });
    }

    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());

    if children.is_empty() {
        quote! { #tag_ident { #(#attr_tokens),* } }
    } else {
        quote! { #tag_ident { #(#attr_tokens),* #(#children)* } }
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);

    if let Some(else_branch) = &if_node.else_branch {
        let else_branch_tokens = emit_nodes(else_branch);
        quote! {
            if #cond {
                dioxus::prelude::rsx! { #then_branch }
            } else {
                dioxus::prelude::rsx! { #else_branch_tokens }
            }
        }
    } else {
        quote! {
            if #cond {
                dioxus::prelude::rsx! { #then_branch }
            }
        }
    }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_render = emit_render(&fe.item_template);

    quote! {
        #items.iter().map(|item| {
            let _key = #key;
            dioxus::prelude::rsx! { #item_render }
        })
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! { #(#node_tokens)* }
}
