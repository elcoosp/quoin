use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::virtual_list_codegen::generate_leptos_virtual_list;
use crate::transpile::rich_text_codegen::generate_leptos_rich_text;
use crate::transpile::dropdown_codegen::{generate_leptos_dropdown, MenuItemDef};
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
    if let Some(custom_tokens) = resolve_custom_element(&name_str) {
        return custom_tokens;
    }
    match name_str.as_str() {
        "virtual_list" => emit_virtual_list(el),
        "rich_text" => emit_rich_text(el),
        "dropdown" => emit_dropdown(el),
        "tabs" => emit_tabs(el),
        _ => emit_builtin(el),
    }
}

fn emit_builtin(el: &Element) -> TokenStream {
    let tag = match el.name.to_string().as_str() {
        "div" => "div", "h1" => "h1", "h2" => "h2", "h3" => "h3",
        "p" | "text" => "p", "button" => "button", "input" => "input", _ => "div",
    };
    let mut attr_tokens = Vec::new();
    for (key, value) in &el.args {
        let key_str = key.to_string();
        match key_str.as_str() {
            "class" => attr_tokens.push(quote! { class=#value }),
            "id" => attr_tokens.push(quote! { id=#value }),
            "on_click" => attr_tokens.push(quote! { on:click=#value }),
            "value" => attr_tokens.push(quote! { value=#value }),
            "placeholder" => attr_tokens.push(quote! { placeholder=#value }),
            _ => {}
        }
    }
    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if children.is_empty() {
        quote! { <#tag_ident #(#attr_tokens)* /> }
    } else {
        quote! { <#tag_ident #(#attr_tokens)*> #(#children)* </#tag_ident> }
    }
}

fn emit_virtual_list(el: &Element) -> TokenStream {
    let items_expr = find_arg_expr(el, "items").expect("virtual_list requires 'items'");
    let estimated_height: f32 = find_arg_lit_string(el, "estimated_height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(32.0);
    let item_template = el.children.first().expect("virtual_list requires item template");
    let item_render = emit_render(item_template);
    generate_leptos_virtual_list(items_expr, estimated_height, item_render)
}

fn emit_rich_text(el: &Element) -> TokenStream {
    let text_expr = find_arg_expr(el, "text").expect("rich_text requires 'text'");
    let base_color = find_arg_expr(el, "base_color");
    let font_size: f32 = find_arg_lit_string(el, "font_size")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(14.0);
    let runs_expr = find_arg_expr(el, "runs");
    generate_leptos_rich_text(text_expr, base_color, font_size, runs_expr)
}

fn emit_dropdown(el: &Element) -> TokenStream {
    let trigger_expr = find_arg_expr(el, "trigger").expect("dropdown requires 'trigger'");
    let menu_items: Vec<MenuItemDef> = el.children.iter().filter_map(|child| {
        if let RenderNode::Element(e) = child {
            if e.name == "menu_item" {
                let label = find_arg_expr(e, "label").expect("menu_item requires 'label'");
                let on_click = find_arg_expr(e, "on_click").expect("menu_item requires 'on_click'");
                return Some(MenuItemDef { label: label.clone(), on_click: on_click.clone() });
            }
        }
        None
    }).collect();
    generate_leptos_dropdown(trigger_expr, &menu_items)
}

fn emit_tabs(el: &Element) -> TokenStream {
    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    quote! { <div> #(#children)* </div> }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = emit_nodes(else_branch);
        quote! { {move || if #cond { view! { #then_branch } } else { view! { #else_tokens } }} }
    } else {
        quote! { {move || if #cond { view! { #then_branch } }} }
    }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_render = emit_render(&fe.item_template);
    quote! { <leptos::prelude::For each=move || #items.clone() key=#key children=move |item| view! { #item_render } /> }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
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
            } else { None }
        } else { None }
    })
}
