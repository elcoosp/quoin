use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;
use crate::render_ast::{RenderNode, Element, IfNode, ForEachNode};
use crate::transpile::tailwind::transpile_class;
use crate::transpile::virtual_list_codegen::generate_gpui_virtual_list;
use crate::transpile::rich_text_codegen::generate_gpui_rich_text;
use crate::transpile::dropdown_codegen::{generate_gpui_dropdown, MenuItemDef};
use crate::transpile::table_codegen::{generate_gpui_table_delegate, ColumnDef};
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
    let name_str = el.name.to_string();
    let mut chain = match name_str.as_str() {
        "div" => quote! { gpui::div() },
        "h1" => quote! { gpui::div().text_xl().font_weight(gpui::FontWeight::BOLD) },
        "h2" => quote! { gpui::div().text_lg().font_weight(gpui::FontWeight::BOLD) },
        "p" | "text" => quote! { gpui::div() },
        "button" => quote! { gpui::div().cursor_pointer() },
        _ => quote! { gpui::div() },
    };
    if let Some((_, class_expr)) = el.args.iter().find(|(k,_)| k == "class") {
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
    if children.is_empty() { chain } else { quote! { #chain #(#children)* } }
}

fn emit_virtual_list(el: &Element) -> TokenStream {
    let items = find_arg_expr(el, "items").expect("virtual_list requires 'items'");
    let estimated_height: f32 = find_arg_lit_string(el, "estimated_height")
        .and_then(|s| s.parse().ok()).unwrap_or(32.0);
    let id = find_arg_lit_string(el, "id").unwrap_or_else(|| "virtual-list".to_string());
    let adapter_name = find_arg_ident(el, "adapter").unwrap_or_else(|| "_vlist_adapter".to_string());
    let item_template = el.children.first().expect("virtual_list requires item template");
    let item_render = emit_render(item_template);
    generate_gpui_virtual_list(items, estimated_height, &id, &adapter_name, item_render)
}

fn emit_rich_text(el: &Element) -> TokenStream {
    let text = find_arg_expr(el, "text").expect("rich_text requires 'text'");
    let base_color = find_arg_expr(el, "base_color");
    let font_size: f32 = find_arg_lit_string(el, "font_size")
        .and_then(|s| s.parse().ok()).unwrap_or(14.0);
    let runs = find_arg_expr(el, "runs");
    generate_gpui_rich_text(text, base_color, font_size, runs)
}

fn emit_dropdown(el: &Element) -> TokenStream {
    let trigger = find_arg_expr(el, "trigger").expect("dropdown requires 'trigger'");
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
    generate_gpui_dropdown(trigger, &items)
}

fn emit_tabs(el: &Element) -> TokenStream {
    let children: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    quote! { gpui::div().flex().child(gpui::div().children(#(#children)*)) }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows").expect("data_table requires 'rows'");
    let striped = find_arg_lit_string(el, "striped")
        .and_then(|s| s.parse().ok()).unwrap_or(false);
    let adapter_name = find_arg_ident(el, "adapter")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "_table_adapter".to_string());
    let columns: Vec<ColumnDef> = el.children.iter().filter_map(|c| {
        if let RenderNode::Element(e) = c {
            if e.name == "column" {
                let key = find_arg_lit_string(e, "key").unwrap_or_default();
                let label = find_arg_lit_string(e, "label").unwrap_or_default();
                let width = find_arg_lit_string(e, "width").and_then(|s| s.parse().ok());
                let sortable = find_arg_lit_string(e, "sortable").and_then(|s| s.parse().ok()).unwrap_or(false);
                let render_closure = find_arg_expr(e, "render")
                    .or_else(|| e.children.first().and_then(|c| if let RenderNode::Expr(expr) = c { Some(expr) } else { None }))
                    .expect("column requires 'render' or a closure child");
                Some(ColumnDef { key, label, width, sortable, render_closure: render_closure.clone() })
            } else { None }
        } else { None }
    }).collect();
    let delegate_str = format!("__QuoinTableDelegate_{}", uuid::Uuid::new_v4().simple());
    let delegate_name = Ident::new(&delegate_str, Span::call_site());
    let row_type = syn::parse_str("_").unwrap();
    let delegate = generate_gpui_table_delegate(&delegate_name, &row_type, &columns);
    quote! {{
        #delegate
        gpui_component::table::DataTable::new(&self.#adapter_name).striped(#striped)
    }}
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    let then_element = quote! { #then_branch.into_any_element() };
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        let else_element = quote! { #else_branch.into_any_element() };
        quote! { if #cond { #then_element } else { #else_element } }
    } else { quote! { if #cond { #then_element } } }
}

fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    let items = &fe.items;
    let key = &fe.key;
    let item_render = emit_render(&fe.item_template);
    quote! { .children({ let items = #items; items.into_iter().map(|item| { let _key = #key; #item_render }) }) }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render).collect();
    quote! { gpui::div() #(#tokens)* }
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
fn find_arg_ident(el: &Element, key: &str) -> Option<String> {
    find_arg_expr(el, key).and_then(|e| {
        if let Expr::Path(expr_path) = e {
            expr_path.path.get_ident().map(|i| i.to_string())
        } else { None }
    })
}
