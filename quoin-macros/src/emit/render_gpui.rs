use proc_macro2::TokenStream;
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::tailwind::transpile_class;
use crate::transpile::virtual_list_codegen::generate_gpui_virtual_list;
use crate::transpile::rich_text_codegen::generate_gpui_rich_text;
use crate::transpile::dropdown_codegen::{generate_gpui_dropdown, MenuItemDef};
use crate::custom_element::resolve_custom_element;
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
    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    if children.is_empty() {
        chain
    } else {
        quote! { #chain #(#children)* }
    }
}

fn emit_virtual_list(el: &Element) -> TokenStream {
    let items_expr = find_arg_expr(el, "items").expect("virtual_list requires 'items'");
    let estimated_height: f32 = find_arg_lit_string(el, "estimated_height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(32.0);
    let id = find_arg_lit_string(el, "id").unwrap_or_else(|| "virtual-list".to_string());
    let adapter_name = find_arg_ident(el, "adapter").unwrap_or_else(|| "_vlist_adapter".to_string());
    let item_template = el.children.first().expect("virtual_list requires item template");
    let item_render = emit_render(item_template);
    generate_gpui_virtual_list(items_expr, estimated_height, &id, &adapter_name, item_render)
}

fn emit_rich_text(el: &Element) -> TokenStream {
    let text_expr = find_arg_expr(el, "text").expect("rich_text requires 'text'");
    let base_color = find_arg_expr(el, "base_color");
    let font_size: f32 = find_arg_lit_string(el, "font_size")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(14.0);
    let runs_expr = find_arg_expr(el, "runs");
    generate_gpui_rich_text(text_expr, base_color, font_size, runs_expr)
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
    generate_gpui_dropdown(trigger_expr, &menu_items)
}

fn emit_tabs(el: &Element) -> TokenStream {
    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    quote! { gpui::div().flex().child(gpui::div().children(#(#children)*)) }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    let then_element = quote! { #then_branch.into_any_element() };
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        let else_element = quote! { #else_branch.into_any_element() };
        quote! { if #cond { #then_element } else { #else_element } }
    } else {
        quote! { if #cond { #then_element } }
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
    quote! { gpui::div() #(#node_tokens)* }
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

fn find_arg_ident(el: &Element, key: &str) -> Option<String> {
    find_arg_expr(el, key).and_then(|e| {
        if let Expr::Path(expr_path) = e {
            expr_path.path.get_ident().map(|i| i.to_string())
        } else { None }
    })
}
