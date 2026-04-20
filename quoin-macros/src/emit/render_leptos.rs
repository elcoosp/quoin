use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::virtual_list_codegen::generate_leptos_virtual_list;
use crate::transpile::rich_text_codegen::generate_leptos_rich_text;
use crate::transpile::dropdown_codegen::{generate_leptos_dropdown, MenuItemDef};
use crate::transpile::table_codegen::{generate_leptos_table, ColumnDef};
use crate::custom_element::resolve_custom_element;
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
    if let Some(custom) = resolve_custom_element(&name_str) { return custom; }
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
        "div" => "div", "h1" => "h1", "h2" => "h2", "h3" => "h3",
        "p"|"text" => "p", "button" => "button", "input" => "input", _ => "div",
    };
    let mut attrs = Vec::new();
    for (k, v) in &el.args {
        match k.to_string().as_str() {
            "class" => attrs.push(quote! { class=#v }),
            "id" => attrs.push(quote! { id=#v }),
            "on_click" => attrs.push(quote! { on:click=#v }),
            "value" => attrs.push(quote! { value=#v }),
            "placeholder" => attrs.push(quote! { placeholder=#v }),
            _ => {}
        }
    }
    let children: Vec<_> = el.children.iter().map(emit_render).collect();
    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if children.is_empty() {
        quote! { <#tag_ident #(#attrs)* /> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children)* </#tag_ident> }
    }
}

fn emit_virtual_list(el: &Element) -> TokenStream {
    let items = find_arg_expr(el, "items").unwrap();
    let height = find_arg_lit_string(el, "estimated_height").and_then(|s| s.parse().ok()).unwrap_or(32.0);
    let template = el.children.first().unwrap();
    let item_render = emit_render(template);
    generate_leptos_virtual_list(items, height, item_render)
}

fn emit_rich_text(el: &Element) -> TokenStream {
    let text = find_arg_expr(el, "text").unwrap();
    let color = find_arg_expr(el, "base_color");
    let size = find_arg_lit_string(el, "font_size").and_then(|s| s.parse().ok()).unwrap_or(14.0);
    let runs = find_arg_expr(el, "runs");
    generate_leptos_rich_text(text, color, size, runs)
}

fn emit_dropdown(el: &Element) -> TokenStream {
    let trigger = find_arg_expr(el, "trigger").unwrap();
    let items: Vec<MenuItemDef> = el.children.iter().filter_map(|c| {
        if let RenderNode::Element(e) = c {
            if e.name == "menu_item" {
                Some(MenuItemDef {
                    label: find_arg_expr(e, "label").unwrap().clone(),
                    on_click: find_arg_expr(e, "on_click").unwrap().clone(),
                })
            } else { None }
        } else { None }
    }).collect();
    generate_leptos_dropdown(trigger, &items)
}

fn emit_tabs(el: &Element) -> TokenStream {
    let children: Vec<_> = el.children.iter().map(emit_render).collect();
    quote! { <div class="tabs"> #(#children)* </div> }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows = find_arg_expr(el, "rows").unwrap();
    let striped = find_arg_lit_string(el, "striped").and_then(|s| s.parse().ok()).unwrap_or(false);
    let columns: Vec<ColumnDef> = el.children.iter().filter_map(|c| {
        if let RenderNode::Element(e) = c {
            if e.name == "column" {
                Some(ColumnDef {
                    key: find_arg_lit_string(e, "key").unwrap_or_default(),
                    label: find_arg_lit_string(e, "label").unwrap_or_default(),
                    width: find_arg_lit_string(e, "width").and_then(|s| s.parse().ok()),
                    sortable: find_arg_lit_string(e, "sortable").and_then(|s| s.parse().ok()).unwrap_or(false),
                    render_closure: find_arg_expr(e, "render").unwrap().clone(),
                })
            } else { None }
        } else { None }
    }).collect();
    let row_type = syn::parse_str("_").unwrap();
    generate_leptos_table(&row_type, &columns, rows, striped)
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = emit_nodes(else_branch);
        quote! { {move || if #cond { view! { #then_tokens } } else { view! { #else_tokens } }} }
    } else {
        quote! { {move || if #cond { view! { #then_tokens } }} }
    }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_render = emit_render(&fe.item_template);
    quote! { <leptos::prelude::For each=move || #items.clone() key=#key children=move |item| view! { #item_render } /> }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render).collect();
    quote! { #(#tokens)* }
}

fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|(k,_)| k == key).map(|(_,v)| v)
}
fn find_arg_lit_string(el: &Element, key: &str) -> Option<String> {
    find_arg_expr(el, key).and_then(|e| {
        if let Expr::Lit(expr_lit) = e {
            if let syn::Lit::Str(s) = &expr_lit.lit { Some(s.value()) } else { None }
        } else { None }
    })
}
