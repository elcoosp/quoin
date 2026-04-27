use crate::emit::common::find_arg_expr;
use crate::render_ast::{Element, RenderNode};
use crate::transpile::dropdown_codegen::{MenuItemDef, generate_gpui_dropdown};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn emit_dropdown_menu(el: &Element) -> TokenStream {
    let trigger_expr = match &el.trigger_expr {
        Some(e) => e,
        None => return quote! { ::gpui::div().child("dropdown: missing trigger") },
    };

    let menu_items: Vec<MenuItemDef> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "item"
            {
                let label = find_arg_expr(e, "label")?;
                let on_click = find_arg_expr(e, "on_click")?;
                return Some(MenuItemDef {
                    label: label.clone(),
                    on_click: on_click.clone(),
                });
            }
            None
        })
        .collect();

    generate_gpui_dropdown(trigger_expr, &menu_items)
}
