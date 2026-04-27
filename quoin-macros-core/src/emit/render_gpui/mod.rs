use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{Element, RenderNode};
use crate::transpile::tailwind::{TranspiledStyles, transpile_class};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

mod button;
mod clipboard_button;
mod control_flow;
mod data_table;
mod dropdown_menu;
mod generic;
mod handler;
mod input;
mod tabs;
mod virtual_list;

pub use self::emit_render as emit_component;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    wrap_with_cfg(node.attrs(), inner)
}

fn emit_render_inner(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { #e },
        RenderNode::If(if_node) => control_flow::emit_if(if_node),
        RenderNode::For(for_node) => control_flow::emit_for(for_node),
        RenderNode::Root(nodes) => emit_nodes(nodes),
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let inner = emit_element_inner(el);
    wrap_with_cfg(&el.attrs, inner)
}

fn emit_element_inner(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let effective_name = match name_str.as_str() {
        "tab_bar" => "tabs",
        other => other,
    };

    match effective_name {
        "button" => button::emit_button(el),
        "input" => input::emit_input(el),
        "tabs" => tabs::emit_tabs(el),
        "data_table" => data_table::emit_data_table(el),
        "virtual_list" => virtual_list::emit_virtual_list(el),
        "dropdown_menu" => dropdown_menu::emit_dropdown_menu(el),
        "clipboard_button" => clipboard_button::emit_clipboard_button(el),
        _ => generic::emit_generic_element(el),
    }
}

pub(crate) fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! { ::gpui::div().children(vec![#(#node_tokens),*]) }
}

pub(crate) fn try_transpile_class(expr: &Expr) -> Option<TranspiledStyles> {
    if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = expr {
        return Some(transpile_class(&s.value()));
    }
    None
}
