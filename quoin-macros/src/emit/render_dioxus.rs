// quoin-macros/src/emit/render_dioxus.rs
use crate::custom_element::resolve_custom_element;
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::dropdown_codegen::{MenuItemDef, generate_dioxus_dropdown};
use crate::transpile::rich_text_codegen::generate_dioxus_rich_text;
use crate::transpile::table_codegen::{ColumnDef, generate_dioxus_table};
use crate::transpile::virtual_list_codegen::generate_dioxus_virtual_list;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    quote! {
        {
            // Must import the re-exported internal crate to satisfy rsx! internals
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
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    if let Some(custom) = resolve_custom_element(&name_str) {
        return custom;
    }
    match name_str.as_str() {
        "virtual_list" => emit_virtual_list(el),
        "rich_text" => emit_rich_text(el),
        "dropdown" => emit_dropdown(el),
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

    for (k, v) in &el.args {
        let key_str = k.to_string();
        match key_str.as_str() {
            "on_click" => items.push(quote! { onclick: #v }),
            "on_input" => items.push(quote! { oninput: #v }),
            "class" => items.push(quote! { class: #v }),
            "id" => items.push(quote! { id: #v }),
            _ => {
                let key = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
                items.push(quote! { #key: #v });
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

fn emit_virtual_list(el: &Element) -> TokenStream {
    let items = find_arg_expr(el, "items").unwrap();
    let height = find_arg_lit_string(el, "estimated_height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(32.0);
    let template = el.children.first().unwrap();
    let item_render = emit_render_inner(template);
    generate_dioxus_virtual_list(items, height, item_render)
}

fn emit_rich_text(el: &Element) -> TokenStream {
    let text = find_arg_expr(el, "text").unwrap();
    let color = find_arg_expr(el, "base_color");
    let size = find_arg_lit_string(el, "font_size")
        .and_then(|s| s.parse().ok())
        .unwrap_or(14.0);
    let runs = find_arg_expr(el, "runs");
    generate_dioxus_rich_text(text, color, size, runs)
}

fn emit_dropdown(el: &Element) -> TokenStream {
    let trigger = find_arg_expr(el, "trigger").unwrap();
    let items: Vec<MenuItemDef> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "menu_item" {
                    Some(MenuItemDef {
                        label: find_arg_expr(e, "label").unwrap().clone(),
                        on_click: find_arg_expr(e, "on_click").unwrap().clone(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    generate_dioxus_dropdown(trigger, &items)
}

fn emit_tabs(_el: &Element) -> TokenStream {
    quote! { div {} }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows = find_arg_expr(el, "rows").unwrap();
    let striped = find_arg_lit_string(el, "striped")
        .and_then(|s| s.parse().ok())
        .unwrap_or(false);
    let columns: Vec<ColumnDef> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    Some(ColumnDef {
                        key: find_arg_lit_string(e, "key").unwrap_or_default(),
                        label: find_arg_lit_string(e, "label").unwrap_or_default(),
                        width: find_arg_lit_string(e, "width").and_then(|s| s.parse().ok()),
                        sortable: find_arg_lit_string(e, "sortable")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(false),
                        render_closure: find_arg_expr(e, "render").unwrap().clone(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    let _row_type = syn::parse_str("_").unwrap();
    generate_dioxus_table(&_row_type, &columns, rows, striped)
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
    el.args.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn find_arg_lit_string(el: &Element, key: &str) -> Option<String> {
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
