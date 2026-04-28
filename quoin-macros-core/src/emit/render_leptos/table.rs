use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

pub(crate) fn emit_table(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    generic::emit_html_tag(el, "table", bindings, inside_for)
}
