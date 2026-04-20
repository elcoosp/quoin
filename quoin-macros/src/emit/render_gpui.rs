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

    // Apply class and other styling attributes (excluding 'children' and 'on_click')
    for (key, value_expr) in &el.args {
        let key_str = key.to_string();
        if key_str == "class" {
            if let Expr::Lit(expr_lit) = value_expr {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let styles = transpile_class(&lit_str.value());
                    for style in styles {
                        chain = quote! { #chain #style };
                    }
                }
            }
        }
        // Other attributes can be added here
    }

    // Check for explicit 'children' attribute (for collections)
    let children_attr = el.args.iter().find(|(k, _)| k == "children");
    if let Some((_, children_expr)) = children_attr {
        chain = quote! { #chain.children(#children_expr) };
    } else {
        // Otherwise, treat each child as a single element and use .child()
        for child in &el.children {
            let child_expr = emit_render(child);
            chain = quote! { #chain.child(#child_expr) };
        }
    }

    // Event handlers
    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_click") {
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, #handler_expr)
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
    quote! {
        {
            let items = #items;
            items.into_iter().map(|item| {
                #item_elem
            }).collect::<Vec<_>>()
        }
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! {
        gpui::div().children(vec![#(#node_tokens),*])
    }
}
