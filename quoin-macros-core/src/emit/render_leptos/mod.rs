#![allow(unused_variables)]

use crate::emit::cfg::wrap_with_cfg;
use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

mod badge;
mod bindings;
mod button;
mod checkbox;
mod clipboard_button;
mod control_flow;
mod data_table;
mod dropdown_menu;
mod generic;
mod handler;
mod icon;
mod input;
mod progress;
mod radio;
mod scroll_area;
mod separator;
mod skeleton;
mod slider;
mod styled_text;
mod switch;
mod tabs;
mod tooltip;
mod virtual_list;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let mut bindings = Vec::new();
    let inner = emit_node(node, &mut bindings, false);

    let tokens = if bindings.is_empty() {
        quote! { { use leptos::prelude::*; leptos::view! { #inner } } }
    } else {
        quote! { { use leptos::prelude::*; #(#bindings)* leptos::view! { #inner } } }
    };

    wrap_with_cfg(node.attrs(), tokens)
}

pub(crate) fn emit_node(node: &RenderNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el, bindings, inside_for),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => {
            if inside_for {
                quote! { {#e} }
            } else {
                quote! { {(#e).clone()} }
            }
        }
        RenderNode::If(if_node) => control_flow::emit_if(if_node, bindings, inside_for),
        RenderNode::For(for_node) => control_flow::emit_for(for_node, bindings),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes
                .iter()
                .map(|n| emit_node(n, bindings, inside_for))
                .collect();
            if tokens.len() == 1 {
                tokens[0].clone()
            } else {
                quote! { <> #(#tokens)* </> }
            }
        }
    }
}

fn emit_element(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = emit_element_inner(el, bindings, inside_for);
    wrap_with_cfg(&el.attrs, inner)
}

fn emit_element_inner(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "separator" => separator::emit_separator(el, bindings, inside_for),
        "skeleton" => skeleton::emit_skeleton(el, bindings, inside_for),
        "skeleton_text" => skeleton::emit_skeleton_text(el, bindings, inside_for),
        "skeleton_avatar" => skeleton::emit_skeleton_avatar(el, bindings, inside_for),
        "progress" => progress::emit_progress(el, bindings, inside_for),
        "checkbox" => checkbox::emit_checkbox(el, bindings, inside_for),
        "switch" => switch::emit_switch(el, bindings, inside_for),
        "radio_group" => radio::emit_radio_group(el, bindings, inside_for),
        "radio" => radio::emit_radio(el, bindings, inside_for),
        "slider" => slider::emit_slider(el, bindings, inside_for),
        "tooltip" => tooltip::emit_tooltip(el, bindings, inside_for),
        "tabs" => tabs::emit_tabs(el, bindings, inside_for),
        "data_table" => data_table::emit_data_table(el, bindings, inside_for),
        "dropdown_menu" => dropdown_menu::emit_dropdown_menu(el, bindings, inside_for),
        "styled_text" => styled_text::emit_styled_text(el, bindings, inside_for),
        "badge" => badge::emit_badge(el, bindings, inside_for),
        "scroll_area" => scroll_area::emit_scroll_area(el, bindings, inside_for),
        "virtual_list" => virtual_list::emit_virtual_list(el, bindings, inside_for),
        "clipboard_button" => clipboard_button::emit_clipboard_button(el, bindings, inside_for),
        "button" => button::emit_button(el, bindings, inside_for),
        "input" => input::emit_input(el, bindings, inside_for),
        "icon" => icon::emit_icon(el, bindings, inside_for),
        _ => generic::emit_html_tag(
            el,
            match name_str.as_str() {
                "div" => "div",
                "h1" => "h1",
                "h2" => "h2",
                "h3" => "h3",
                "p" | "text" => "p",
                "span" => "span",
                "a" => "a",
                "ul" => "ul",
                "li" => "li",
                "label" => "label",
                "textarea" => "textarea",
                "select" => "select",
                "form" => "form",
                "img" => "img",
                _ => "div",
            },
            bindings,
            inside_for,
        ),
    }
}
