use crate::emit::common::find_arg_string;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::import_shadcn_or_html_tag;

pub(crate) fn resolve_separator_element(
    bindings: &mut Vec<TokenStream>,
    el: &Element,
) -> proc_macro2::Ident {
    let orientation =
        find_arg_string(el, "orientation").unwrap_or_else(|| "horizontal".to_string());
    let html_tag = if orientation == "horizontal" {
        "hr"
    } else {
        "div"
    };
    import_shadcn_or_html_tag(bindings, "Separator", html_tag)
}

pub(crate) fn emit_separator(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = resolve_separator_element(bindings, el);
    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        if key_str == "class" {
            attrs.push(quote! { class=#value });
        }
        // `orientation` was consumed in `resolve_separator_element`; the
        // remaining attributes are ignored.
    }
    if attrs.is_empty() {
        quote! { <#tag /> }
    } else {
        quote! { <#tag #(#attrs)* /> }
    }
}
