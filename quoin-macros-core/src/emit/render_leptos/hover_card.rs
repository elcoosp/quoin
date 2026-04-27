use crate::render_ast::Element;
use proc_macro2::TokenStream;
use super::emit_node;

pub(crate) fn emit_hover_card(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        // TODO: implement ShadCN emission
        super::generic::emit_html_tag_inner(el, "div", bindings, inside_for)
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        super::generic::emit_html_tag_inner(el, "div", bindings, inside_for)
    }
}
