use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { {#e} },
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

    let mut attrs = Vec::new();

    for (key, value) in &el.args {
        let key_str = key.to_string();
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "on_click" => {
                attrs.push(quote! { on:click=move |_| { #value } });
            }
            _ => {}
        }
    }

    let mut children_tokens = Vec::new();

    if let Some(children_expr) = &el.children_expr {
        children_tokens.push(quote! { {#children_expr} });
    } else {
        for child in &el.children {
            children_tokens.push(emit_render(child));
        }
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if children_tokens.is_empty() {
        quote! { <#tag_ident #(#attrs)* /> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children_tokens)* </#tag_ident> }
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        quote! { {move || if #cond { #then_branch } else { #else_branch }} }
    } else {
        quote! { {move || if #cond { #then_branch }} }
    }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_elem = emit_render(&fe.item_template);
    quote! {
        {
            let items = #items;
            items.into_iter().map(|item| {
                let _key = #key;
                #item_elem
            }).collect::<Vec<_>>()
        }
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render).collect();
    quote! { #(#tokens)* }
}
