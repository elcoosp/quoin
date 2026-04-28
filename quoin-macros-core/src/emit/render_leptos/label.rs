use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

pub(crate) fn emit_label(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class_expr = find_arg_expr(el, "class");
        let class_prop = class_expr.map(|c| quote! { class={ #c.into() } }).unwrap_or_else(|| quote! {});
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let alias = quote::format_ident!("Label_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Label; });
        if children.is_empty() {
            quote! { <#alias #class_prop /> }
        } else {
            quote! { <#alias #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        generic::emit_html_tag(el, "label", bindings, inside_for)
    }
}
