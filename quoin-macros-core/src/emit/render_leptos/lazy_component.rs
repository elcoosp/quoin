use crate::emit::common::find_arg_string;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

pub(crate) fn emit_lazy_component(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let name = find_arg_string(el, "name").unwrap_or_default();
        let class = find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("LazyComponent_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::LazyComponent; });
        quote! { <#alias name={#name} #class_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
