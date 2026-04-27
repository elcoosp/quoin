use crate::emit::common::find_arg_f32;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::emit_render_inner;

// WARNING: This is a stub implementation that does NOT provide true virtualization.
// All child elements are rendered into a scrollable container regardless of the
// number of items. The `estimated_height` parameter only sets the container's
// fixed height via CSS but does NOT affect which items are rendered.
pub(crate) fn emit_virtual_list(el: &Element) -> TokenStream {
    let estimated_height = find_arg_f32(el, "estimated_height");
    let children_tokens: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    let style = match estimated_height {
        Some(h) => format!("overflow-y: auto; height: {}px", h),
        None => "overflow-y: auto".to_string(),
    };
    quote! { div { style: #style, #(#children_tokens)* } }
}
