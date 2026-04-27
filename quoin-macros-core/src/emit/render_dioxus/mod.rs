use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::collect_handler_idents_excluding_params;
use proc_macro2::TokenStream;
use quote::quote;

mod button;
mod control_flow;
mod data_table;
mod dropdown_menu;
mod form_controls;
mod generic;
mod handler;
mod input;
mod simple_components;
mod tabs;
mod tier1_components;
mod tooltip;
mod virtual_list;

// ---------------------------------------------------------------------------
// Top-level render entry point
// ---------------------------------------------------------------------------
pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    let tokens = quote! {
        {
            use dioxus::prelude::*;
            dioxus::prelude::rsx! { #inner }
        }
    };
    wrap_with_cfg(node_attrs(node), tokens)
}

fn node_attrs(node: &RenderNode) -> &[syn::Attribute] {
    match node {
        RenderNode::Element(el) => &el.attrs,
        RenderNode::If(if_node) => &if_node.attrs,
        RenderNode::For(for_node) => &for_node.attrs,
        RenderNode::Text(_) | RenderNode::Expr(_) | RenderNode::Root(_) => &[],
    }
}

pub(crate) fn emit_render_inner(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { {#e} },
        RenderNode::If(if_node) => control_flow::emit_if(if_node),
        RenderNode::For(for_node) => control_flow::emit_for(for_node),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes.iter().map(emit_render_inner).collect();
            quote! { #(#tokens)* }
        }
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
        "separator" => tier1_components::emit_separator(el),
        "skeleton" => tier1_components::emit_skeleton(el),
        "skeleton_text" => tier1_components::emit_skeleton_text(el),
        "skeleton_avatar" => tier1_components::emit_skeleton_avatar(el),
        "progress" => tier1_components::emit_progress(el),
        "checkbox" => form_controls::emit_checkbox(el),
        "switch" => form_controls::emit_switch(el),
        "radio_group" => form_controls::emit_radio_group(el),
        "radio" => form_controls::emit_radio(el),
        "slider" => form_controls::emit_slider(el),
        "tooltip" => tooltip::emit_tooltip(el),
        "tabs" => tabs::emit_tabs(el),
        "data_table" => data_table::emit_data_table(el),
        "dropdown_menu" => dropdown_menu::emit_dropdown_menu(el),
        "styled_text" => simple_components::emit_styled_text(el),
        "badge" => simple_components::emit_badge(el),
        "scroll_area" => simple_components::emit_scroll_area(el),
        "virtual_list" => virtual_list::emit_virtual_list(el),
        "clipboard_button" => simple_components::emit_clipboard_button(el),
        "button" => button::emit_button(el),
        "icon" => simple_components::emit_icon(el),
        "input" => input::emit_input(el),
        _ => generic::emit_html_el(el, &name_str),
    }
}

pub(crate) fn emit_nodes_inner(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render_inner).collect();
    quote! { #(#tokens)* }
}
