use crate::emit::common::find_arg_string;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::import_shadcn_or_html_tag;

pub(crate) fn emit_skeleton(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse rounded-md bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}

pub(crate) fn emit_skeleton_text(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse h-4 w-full rounded-md bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}

pub(crate) fn emit_skeleton_avatar(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse h-10 w-10 rounded-full bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}
