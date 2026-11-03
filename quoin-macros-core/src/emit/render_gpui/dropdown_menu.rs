use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::handler::wrap_handler;

/// Emits a `dropdown_menu(...)` element as a `gpui_component::Button` with a
/// real `gpui_component::PopupMenu` dropdown.
///
/// Supported framework-agnostic arguments (trigger):
/// - `label: &str` (or children) — trigger button text
/// - `ghost: bool`, `small: bool`, `xsmall: bool` — trigger button styling
/// - children: `item(label: "…", checked: <bool expr>, on_click: |…| …)`
pub(crate) fn emit_dropdown_menu(el: &Element) -> TokenStream {
    let label = find_arg_string(el, "label");
    let ghost = find_arg_bool(el, "ghost");
    let small = find_arg_bool(el, "small");
    let xsmall = find_arg_bool(el, "xsmall");

    let mut trigger = quote! { ::gpui_component::button::Button::new("dropdown-trigger") };
    if ghost {
        trigger = quote! { #trigger.ghost() };
    }
    if xsmall {
        trigger = quote! { #trigger.xsmall() };
    } else if small {
        trigger = quote! { #trigger.small() };
    }
    if let Some(label) = &label {
        trigger = quote! { #trigger.label(#label) };
    }

    let items: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "item"
            {
                let label = find_arg_expr(e, "label")?;
                let on_click = find_arg_expr(e, "on_click")?;
                // `PopupMenuItem::on_click` expects `Fn(&ClickEvent, &mut
                // Window, &mut App)`; adapt the user's handler the same way
                // buttons do.
                let on_click = wrap_handler(on_click);
                let checked = find_arg_expr(e, "checked");
                let checked_tokens = checked
                    .map(|expr| quote! { .checked(#expr) })
                    .unwrap_or_default();
                return Some(quote! {
                    __menu = __menu.item(
                        ::gpui_component::menu::PopupMenuItem::new(#label)
                            #checked_tokens
                            .on_click(#on_click),
                    );
                });
            }
            None
        })
        .collect();

    quote! {
        {
            // `dropdown_menu` is a `gpui_component::menu::DropdownMenu`
            // trait method; the trait must be in scope. The callback owns
            // the `PopupMenu` and returns it, so the binding must be `mut`.
            use ::gpui_component::menu::DropdownMenu as _;
            #trigger
                .dropdown_menu(move |mut __menu, _: &mut ::gpui::Window, _: &mut ::gpui::Context<::gpui_component::menu::PopupMenu>| {
                    #(#items)*
                    __menu
                })
        }
    }
}
