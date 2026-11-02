use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use crate::transpile::force_move_on_closure;
use proc_macro2::TokenStream;
use quote::quote;

/// Emits a `tabs(...)` element as a real `gpui_component::TabBar` with
/// `Tab` children.
///
/// Supported framework-agnostic arguments:
/// - `active: usize` — currently selected tab index
/// - `on_click: |index: usize| …` — selection handler (shadow-clones signals)
/// - `id: &str` — optional element id (defaults to `tabs`)
/// - children: `tab(label: "…", icon: "icon-name")`
pub(crate) fn emit_tabs(el: &Element) -> TokenStream {
    let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
    let on_click_expr = find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");
    let on_click_with_move = force_move_on_closure(on_click_expr);
    let id = find_arg_string(el, "id").unwrap_or_else(|| "tabs".to_string());

    let mut tabs: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        if let RenderNode::Element(e) = child
            && e.name == "tab"
        {
            let label = find_arg_string(e, "label").unwrap_or_default();
            let prefix = match find_arg_string(e, "icon") {
                Some(icon) => {
                    let resolved = crate::transpile::icon_codegen::icon_variant_ident(&icon);
                    quote! {
                        .prefix(::gpui_component::Icon::new(::gpui_component::IconName::#resolved).ml_1())
                    }
                }
                None => quote! {},
            };
            tabs.push(quote! {
                ::gpui_component::tab::Tab::new().label(#label) #prefix
            });
        }
    }

    quote! {
        {
            let __active: usize = #active_expr;
            let __on_click = ::std::rc::Rc::new(#on_click_with_move);
            let mut __bar = ::gpui_component::tab::TabBar::new(#id).selected_index(__active);
            __bar = __bar.on_click(move |__selected: &usize, _: &mut ::gpui::Window, _: &mut ::gpui::App| {
                (__on_click)(*__selected);
            });
            #(
                __bar = __bar.child(#tabs);
            )*
            __bar
        }
    }
}
