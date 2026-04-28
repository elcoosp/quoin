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
mod accordion;
mod alert;
mod alert_dialog;
mod avatar;
mod breadcrumb;
mod calendar;
mod card;
mod carousel;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod date_picker;
mod dialog;
mod drawer;
mod form;
mod hover_card;
mod label;
mod other;
mod pagination;
mod sheet;
mod table;
mod select;
mod toast;
mod error_boundary;
mod lazy_component;

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
            if inside_for { quote! { {#e} } } else { quote! { {(#e).clone()} } }
        }
        RenderNode::If(if_node) => control_flow::emit_if(if_node, bindings, inside_for),
        RenderNode::For(for_node) => control_flow::emit_for(for_node, bindings),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes.iter().map(|n| emit_node(n, bindings, inside_for)).collect();
            if tokens.len() == 1 { tokens[0].clone() }
            else { quote! { <> #(#tokens)* </> } }
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
        // --- New ShadCN components (Chunks 1‑3) ---
        "accordion" => accordion::emit_accordion(el, bindings, inside_for),
        "accordion_item" => accordion::emit_accordion_item(el, bindings, inside_for),
        "accordion_trigger" => accordion::emit_accordion_trigger(el, bindings, inside_for),
        "accordion_content" => accordion::emit_accordion_content(el, bindings, inside_for),
        "alert" => alert::emit_alert(el, bindings, inside_for),
        "alert_dialog" => alert_dialog::emit_alert_dialog(el, bindings, inside_for),
        "alert_dialog_trigger" => alert_dialog::emit_alert_dialog_trigger(el, bindings, inside_for),
        "alert_dialog_overlay" => alert_dialog::emit_alert_dialog_overlay(el, bindings, inside_for),
        "alert_dialog_header" => alert_dialog::emit_alert_dialog_header(el, bindings, inside_for),
        "alert_dialog_footer" => alert_dialog::emit_alert_dialog_footer(el, bindings, inside_for),
        "alert_dialog_title" => alert_dialog::emit_alert_dialog_title(el, bindings, inside_for),
        "alert_dialog_description" => alert_dialog::emit_alert_dialog_description(el, bindings, inside_for),
        "alert_dialog_action" => alert_dialog::emit_alert_dialog_action(el, bindings, inside_for),
        "alert_dialog_cancel" => alert_dialog::emit_alert_dialog_cancel(el, bindings, inside_for),
        "avatar" => avatar::emit_avatar(el, bindings, inside_for),
        "avatar_image" => avatar::emit_avatar_image(el, bindings, inside_for),
        "avatar_fallback" => avatar::emit_avatar_fallback(el, bindings, inside_for),
        "avatar_group" => avatar::emit_avatar_group(el, bindings, inside_for),
        "breadcrumb" => breadcrumb::emit_breadcrumb(el, bindings, inside_for),
        "breadcrumb_list" => breadcrumb::emit_breadcrumb_list(el, bindings, inside_for),
        "breadcrumb_item" => breadcrumb::emit_breadcrumb_item(el, bindings, inside_for),
        "breadcrumb_link" => breadcrumb::emit_breadcrumb_link(el, bindings, inside_for),
        "breadcrumb_page" => breadcrumb::emit_breadcrumb_page(el, bindings, inside_for),
        "breadcrumb_separator" => breadcrumb::emit_breadcrumb_separator(el, bindings, inside_for),
        "breadcrumb_ellipsis" => breadcrumb::emit_breadcrumb_ellipsis(el, bindings, inside_for),
        "calendar" => calendar::emit_calendar(el, bindings, inside_for),
        "card" => card::emit_card(el, bindings, inside_for),
        "card_header" => card::emit_card_header(el, bindings, inside_for),
        "card_title" => card::emit_card_title(el, bindings, inside_for),
        "card_description" => card::emit_card_description(el, bindings, inside_for),
        "card_content" => card::emit_card_content(el, bindings, inside_for),
        "card_footer" => card::emit_card_footer(el, bindings, inside_for),
        "carousel" => carousel::emit_carousel(el, bindings, inside_for),
        "collapsible" => collapsible::emit_collapsible(el, bindings, inside_for),
        "combobox" => combobox::emit_combobox(el, bindings, inside_for),
        "command" => command::emit_command(el, bindings, inside_for),
        "command_input" => command::emit_command_input(el, bindings, inside_for),
        "command_list" => command::emit_command_list(el, bindings, inside_for),
        "command_empty" => command::emit_command_empty(el, bindings, inside_for),
        "command_group" => command::emit_command_group(el, bindings, inside_for),
        "command_group_heading" => command::emit_command_group_heading(el, bindings, inside_for),
        "command_item" => command::emit_command_item(el, bindings, inside_for),
        "command_shortcut" => command::emit_command_shortcut(el, bindings, inside_for),
        "command_separator" => command::emit_command_separator(el, bindings, inside_for),
        "context_menu" => context_menu::emit_context_menu(el, bindings, inside_for),
        "context_menu_trigger" => context_menu::emit_context_menu_trigger(el, bindings, inside_for),
        "context_menu_content" => context_menu::emit_context_menu_content(el, bindings, inside_for),
        "context_menu_item" => context_menu::emit_context_menu_item(el, bindings, inside_for),
        "context_menu_separator" => context_menu::emit_context_menu_separator(el, bindings, inside_for),
        "context_menu_label" => context_menu::emit_context_menu_label(el, bindings, inside_for),
        "context_menu_checkbox_item" => context_menu::emit_context_menu_checkbox_item(el, bindings, inside_for),
        "context_menu_radio_group" => context_menu::emit_context_menu_radio_group(el, bindings, inside_for),
        "context_menu_radio_item" => context_menu::emit_context_menu_radio_item(el, bindings, inside_for),
        "context_menu_sub" => context_menu::emit_context_menu_sub(el, bindings, inside_for),
        "context_menu_sub_content" => context_menu::emit_context_menu_sub_content(el, bindings, inside_for),
        "context_menu_sub_trigger" => context_menu::emit_context_menu_sub_trigger(el, bindings, inside_for),
        "context_menu_shortcut" => context_menu::emit_context_menu_shortcut(el, bindings, inside_for),
        "date_picker" => date_picker::emit_date_picker(el, bindings, inside_for),
        "dialog" => dialog::emit_dialog(el, bindings, inside_for),
        "dialog_trigger" => dialog::emit_dialog_trigger(el, bindings, inside_for),
        "dialog_content" => dialog::emit_dialog_content(el, bindings, inside_for),
        "dialog_header" => dialog::emit_dialog_header(el, bindings, inside_for),
        "dialog_title" => dialog::emit_dialog_title(el, bindings, inside_for),
        "dialog_description" => dialog::emit_dialog_description(el, bindings, inside_for),
        "dialog_footer" => dialog::emit_dialog_footer(el, bindings, inside_for),
        "dialog_close" => dialog::emit_dialog_close(el, bindings, inside_for),
        "drawer" => drawer::emit_drawer(el, bindings, inside_for),
        "drawer_trigger" => drawer::emit_drawer_trigger(el, bindings, inside_for),
        "drawer_content" => drawer::emit_drawer_content(el, bindings, inside_for),
        "drawer_overlay" => drawer::emit_drawer_overlay(el, bindings, inside_for),
        "drawer_portal" => drawer::emit_drawer_portal(el, bindings, inside_for),
        "drawer_header" => drawer::emit_drawer_header(el, bindings, inside_for),
        "drawer_footer" => drawer::emit_drawer_footer(el, bindings, inside_for),
        "drawer_title" => drawer::emit_drawer_title(el, bindings, inside_for),
        "drawer_description" => drawer::emit_drawer_description(el, bindings, inside_for),
        "drawer_close" => drawer::emit_drawer_close(el, bindings, inside_for),
        "form" => form::emit_form(el, bindings, inside_for),
        "form_field" => form::emit_form_field(el, bindings, inside_for),
        "form_item" => form::emit_form_item(el, bindings, inside_for),
        "form_label" => form::emit_form_label(el, bindings, inside_for),
        "form_control" => form::emit_form_control(el, bindings, inside_for),
        "form_message" => form::emit_form_message(el, bindings, inside_for),
        "form_description" => form::emit_form_description(el, bindings, inside_for),
        "hover_card" => hover_card::emit_hover_card(el, bindings, inside_for),
        "label" => label::emit_label(el, bindings, inside_for),
        "menubar" => other::emit_menubar(el, bindings, inside_for),
        "navigation_menu" => other::emit_navigation_menu(el, bindings, inside_for),
        "pagination" => pagination::emit_pagination(el, bindings, inside_for),
        "pagination_content" => pagination::emit_pagination_content(el, bindings, inside_for),
        "pagination_item" => pagination::emit_pagination_item(el, bindings, inside_for),
        "pagination_link" => pagination::emit_pagination_link(el, bindings, inside_for),
        "pagination_previous" => pagination::emit_pagination_previous(el, bindings, inside_for),
        "pagination_next" => pagination::emit_pagination_next(el, bindings, inside_for),
        "pagination_ellipsis" => pagination::emit_pagination_ellipsis(el, bindings, inside_for),
        "popover" => other::emit_popover(el, bindings, inside_for),
        "resizable_panel_group" => other::emit_resizable(el, bindings, inside_for),
"select" => select::emit_select(el, bindings, inside_for),
        "select_trigger" => select::emit_select_trigger(el, bindings, inside_for),
        "select_content" => select::emit_select_content(el, bindings, inside_for),
        "select_item" => select::emit_select_item(el, bindings, inside_for),
        "sheet" => sheet::emit_sheet(el, bindings, inside_for),
        "sheet_trigger" => sheet::emit_sheet_trigger(el, bindings, inside_for),
        "sheet_content" => sheet::emit_sheet_content(el, bindings, inside_for),
        "sheet_title" => sheet::emit_sheet_title(el, bindings, inside_for),
        "sheet_description" => sheet::emit_sheet_description(el, bindings, inside_for),
        "table" => table::emit_table(el, bindings, inside_for),
        "textarea" => other::emit_textarea(el, bindings, inside_for),
        "toggle" => other::emit_toggle(el, bindings, inside_for),
"error_boundary" => error_boundary::emit_error_boundary(el, bindings, inside_for),
"lazy_component" => lazy_component::emit_lazy_component(el, bindings, inside_for),
"toast_provider" => toast::emit_toast_provider(el, bindings, inside_for),
        "input_otp" => other::emit_input_otp(el, bindings, inside_for),
        _ => generic::emit_html_tag(
            el,
            match name_str.as_str() {
                "div" => "div", "h1" => "h1", "h2" => "h2", "h3" => "h3",
                "p" | "text" => "p",
                "span" => "span", "a" => "a", "ul" => "ul", "li" => "li",
                "label" => "label", "textarea" => "textarea", "select" => "select",
                "form" => "form", "img" => "img",
                _ => "div",
            },
            bindings,
            inside_for,
        ),
    }
}
