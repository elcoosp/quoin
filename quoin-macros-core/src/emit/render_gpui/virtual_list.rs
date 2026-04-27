use crate::emit::common::{find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use crate::transpile::virtual_list_codegen::generate_gpui_virtual_list;
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_render;

pub(crate) fn emit_virtual_list(el: &Element) -> TokenStream {
    let items_expr = find_arg_expr(el, "items").expect("virtual_list requires 'items:' argument");
    let estimated_height = find_arg_f32(el, "estimated_height")
        .unwrap_or(32.0);
    let id_expr = find_arg_string(el, "id")
        .unwrap_or_else(|| "virtual-list".to_string());

    let item_render_tokens: Vec<TokenStream> = el.children.iter().map(emit_render).collect();
    let item_render =
        quote! { ::gpui::div().children(vec![#(#item_render_tokens),*]).into_any_element() };

    generate_gpui_virtual_list(items_expr, estimated_height, &id_expr, item_render)
}
