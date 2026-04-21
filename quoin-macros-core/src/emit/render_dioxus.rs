use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    quote! {
        {
            use dioxus::prelude::dioxus_elements;
            let __quoin_node: dioxus::prelude::Element = dioxus::prelude::rsx! {
                #inner
            };
            __quoin_node
        }
    }
}

fn emit_render_inner(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { {#e} },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes.iter().map(emit_render_inner).collect();
            quote! { #(#tokens)* }
        }
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el),
        _ => emit_builtin(el),
    }
}

fn emit_builtin(el: &Element) -> TokenStream {
    let tag = match el.name.to_string().as_str() {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        "button" => "button",
        "input" => "input",
        _ => "div",
    };

    let mut items = Vec::new();

    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "on_click" => items.push(quote! { onclick: #value }),
            "on_input" => items.push(quote! { oninput: #value }),
            "class" => items.push(quote! { class: #value }),
            "id" => items.push(quote! { id: #value }),
            _ => {
                let key = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
                items.push(quote! { #key: #value });
            }
        }
    }

    if let Some(children_expr) = &el.children_expr {
        items.push(quote! { {#children_expr.into_iter()} });
    }

    for child in &el.children {
        items.push(emit_render_inner(child));
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());

    if items.is_empty() {
        quote! { #tag_ident {} }
    } else {
        quote! { #tag_ident { #(#items),* } }
    }
}

fn emit_tabs(_el: &Element) -> TokenStream {
    quote! { div {} }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows = find_arg_expr(el, "rows").unwrap();
    let header_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let label = find_arg_string(e, "label").unwrap_or_default();
                    return Some(quote! { th { #label } });
                }
            }
            None
        })
        .collect();

    let row_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let render_closure = find_arg_expr(e, "render").unwrap();
                    return Some(quote! { td { (#render_closure)(&__row) } });
                }
            }
            None
        })
        .collect();

    quote! {
        table {
            thead {
                tr { #(#header_cells)* }
            }
            tbody {
                #rows.iter().map(|__row| {
                    rsx! { tr { #(#row_cells)* } }
                })
            }
        }
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens = emit_nodes_inner(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = emit_nodes_inner(else_branch);
        quote! { if #cond { #then_tokens } else { #else_tokens } }
    } else {
        quote! { if #cond { #then_tokens } }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes_inner(&for_node.body);
    quote! { {#iterable.into_iter().map(|#pat| #body)} }
}

fn emit_nodes_inner(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render_inner).collect();
    quote! { #(#tokens)* }
}

fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|a| a.key == key).map(|a| &a.value)
}

fn find_arg_string(el: &Element, key: &str) -> Option<String> {
    find_arg_expr(el, key).and_then(|e| {
        if let Expr::Lit(expr_lit) = e {
            if let syn::Lit::Str(s) = &expr_lit.lit {
                Some(s.value())
            } else {
                None
            }
        } else {
            None
        }
    })
}
