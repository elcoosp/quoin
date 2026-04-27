use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_node;

pub(crate) fn emit_icon(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let name = find_arg_string(el, "icon_name");

    let size_class = find_arg_expr(el, "class");
    let class_str = match size_class {
        Some(c) => quote! { format!("{} w-4 h-4 inline-block", #c) },
        None => quote! { "w-4 h-4 inline-block" },
    };
    let children: Vec<TokenStream> = el
        .children
        .iter()
        .map(|c| emit_node(c, bindings, inside_for))
        .collect();

    match name {
        Some(n) => {
            if let Some(svg) = crate::transpile::icon_codegen::icon_svg_html(&n) {
                quote! {
                    <span class=#class_str>
                        #svg
                    </span>
                }
            } else {
                quote! {
                    <span class=#class_str>"❓"</span>
                }
            }
        }
        None => {
            if children.is_empty() {
                quote! {
                    <span class=#class_str>"❓"</span>
                }
            } else {
                quote! {
                    <span class=#class_str>
                        #(#children)*
                    </span>
                }
            }
        }
    }
}
