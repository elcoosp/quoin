use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use crate::transpile::force_move_on_closure;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn emit_tabs(el: &Element) -> TokenStream {
    let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
    let on_click_expr = find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");
    let on_click_with_move = force_move_on_closure(on_click_expr);

    let tab_labels: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "tab"
            {
                let label = find_arg_string(e, "label").unwrap_or_default();
                let index = find_arg_expr(e, "index").expect("tab requires 'index'");
                return Some(quote! { ( #index, #label.to_string() ) });
            }
            None
        })
        .collect();

    quote! {
        {
            let __active = #active_expr;
            let __on_click = ::std::rc::Rc::new(#on_click_with_move);
            let __labels: Vec<(usize, String)> = vec![#(#tab_labels),*];
            let __tab_elements: Vec<::gpui::AnyElement> = __labels.iter().map(|(idx, label)| {
                let __is_active = *idx == __active;
                let mut __el = ::gpui::div().px(::gpui::px(16.0)).py(::gpui::px(8.0)).cursor_pointer().child(label.clone());
                if __is_active { __el = __el.text_color(::gpui::white()); }
                else { __el = __el.text_color(::gpui::rgb(0x9ca3af)); }
                let __idx = *idx;
                let __tab_on_click = __on_click.clone();
                __el.on_mouse_down(::gpui::MouseButton::Left,
                    move |_, _, _| { __tab_on_click(__idx) }
                ).into_any_element()
            }).collect();
            ::gpui::div().flex().children(__tab_elements)
        }
    }
}
