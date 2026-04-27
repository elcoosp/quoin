use crate::emit::common::find_arg_f32;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_node;

// WARNING: This is a stub implementation that does NOT provide true virtualization.
pub(crate) fn emit_virtual_list(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let estimated_height = find_arg_f32(el, "estimated_height");
    let children_tokens: Vec<TokenStream> = el
        .children
        .iter()
        .map(|c| emit_node(c, bindings, inside_for))
        .collect();

    let style = match estimated_height {
        Some(h) => format!("overflow-y: auto; height: {}px", h),
        None => "overflow-y: auto".to_string(),
    };
    quote! { <div style=#style> #(#children_tokens)* </div> }
}
