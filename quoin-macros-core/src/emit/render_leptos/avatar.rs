use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

pub(crate) fn emit_avatar(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("Avatar_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Avatar; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_avatar_image(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let src = crate::emit::common::find_arg_expr(el, "src")
            .map(|s| quote! { src={ #s.into() } })
            .unwrap_or_else(|| quote! {});
        let alt = crate::emit::common::find_arg_string(el, "alt")
            .map(|a| quote! { alt={ #a.into() } })
            .unwrap_or_else(|| quote! {});
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("AvatarImage_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AvatarImage; });
        quote! { <#alias #src #alt #class_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "img", bindings, _inside_for) }
}

pub(crate) fn emit_avatar_fallback(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("AvatarFallback_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AvatarFallback; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_avatar_group(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("AvatarGroup_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AvatarGroup; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}
