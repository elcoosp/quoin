// quoin-macros/src/emit/render_gpui.rs
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::tailwind::transpile_class;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { #e },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
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
    if let Some((_, class_expr)) = el.args.iter().find(|(k, _)| k == "class") {
        if let Expr::Lit(expr_lit) = class_expr {
            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                let styles = transpile_class(&lit_str.value());
                for style in styles {
                    chain = quote! { #chain #style };
                }
            }
        }
    }
    if let Some(children_expr) = &el.children_expr {
        chain = quote! { #chain.children(#children_expr) };
    } else {
        for child in &el.children {
            let child_elem = emit_render(child);
            chain = quote! { #chain.child(#child_elem) };
        }
    }
    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_click") {
        chain = quote! { #chain.on_mouse_down(gpui::MouseButton::Left, #handler_expr) };
    }
    chain
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        quote! { if #cond { #then_branch } else { #else_branch } }
    } else {
        quote! { if #cond { #then_branch } }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes(&for_node.body);
    quote! {
        .children(
            #iterable.into_iter().map(|#pat| {
                #body
            })
        )
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<_> = nodes.iter().map(emit_render).collect();
    quote! { gpui::div().children(vec![#(#node_tokens),*]) }
}
