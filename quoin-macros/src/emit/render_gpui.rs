use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::tailwind::transpile_class;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { #e },
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
        let child_exprs: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
        for child in child_exprs {
            chain = quote! { #chain.child(#child) };
        }
    }
    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_click") {
        // Wrap user closure with cx.listener signature
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, cx.listener(move |this, _ev, _window, _cx| {
                let handler = #handler_expr;
                handler(this);
            }))
        };
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

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let _key = &fe.key;
    let item_elem = emit_render(&fe.item_template);
    quote! {{
        let items = #items;
        items.into_iter().map(|item| #item_elem).collect::<Vec<_>>()
    }}
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! { gpui::div().children(vec![#(#node_tokens),*]) }
}
