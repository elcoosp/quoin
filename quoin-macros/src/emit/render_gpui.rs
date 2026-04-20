use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::tailwind::transpile_class;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { .child(#t) },
        RenderNode::Expr(e) => quote! { .child(#e) },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::ForEach(fe) => emit_for_each(fe),
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let mut chain = match name_str.as_str() {
        "div" => quote! { gpui::div() },
        "h1" => quote! { gpui::div().text_xl().font_weight(gpui::FontWeight::BOLD) },
        "h2" => quote! { gpui::div().text_lg().font_weight(gpui::FontWeight::BOLD) },
        "p" | "text" => quote! { gpui::div() },
        "button" => quote! { gpui::div().cursor_pointer() },
        _ => quote! { gpui::div() },
    };

    if let Some((_, class_expr)) = el.args.iter().find(|(k, _)| k.to_string() == "class") {
        if let Expr::Lit(expr_lit) = class_expr {
            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                let styles = transpile_class(&lit_str.value());
                for style in styles {
                    chain = quote! { #chain #style };
                }
            }
        }
    }

    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();

    if children.is_empty() {
        chain
    } else {
        quote! {
            #chain
            #(#children)*
        }
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    let then_element = quote! { #then_branch.into_any_element() };

    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        let else_element = quote! { #else_branch.into_any_element() };
        quote! {
            if #cond {
                #then_element
            } else {
                #else_element
            }
        }
    } else {
        quote! {
            if #cond {
                #then_element
            }
        }
    }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_render = emit_render(&fe.item_template);
    quote! {
        .children({
            let items = #items;
            items.into_iter().map(|item| {
                let _key = #key;
                #item_render
            })
        })
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! {
        gpui::div()
        #(#node_tokens)*
    }
}
