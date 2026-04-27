use crate::emit::common::find_arg_bool;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

fn map_variant(variant_str: &str) -> TokenStream {
    match variant_str {
        "destructive" => quote! { leptos_shadcn_ui::CardVariant::Destructive },
        "warning"     => quote! { leptos_shadcn_ui::CardVariant::Warning },
        "success"     => quote! { leptos_shadcn_ui::CardVariant::Success },
        _             => quote! { leptos_shadcn_ui::CardVariant::Default },
    }
}

pub(crate) fn emit_card(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let variant = crate::emit::common::find_arg_string(el, "variant")
            .map(|v| map_variant(&v))
            .unwrap_or_else(|| quote! { leptos_shadcn_ui::CardVariant::Default });
        let interactive = find_arg_bool(el, "interactive");
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();

        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("Card_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Card; });

        if children.is_empty() {
            quote! { <#alias variant={#variant} interactive={#interactive} #class_prop /> }
        } else {
            quote! { <#alias variant={#variant} interactive={#interactive} #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_card_header(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("CardHeader_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CardHeader; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_card_title(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("CardTitle_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CardTitle; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "h3", bindings, inside_for) }
}

pub(crate) fn emit_card_description(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("CardDescription_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CardDescription; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "p", bindings, inside_for) }
}

pub(crate) fn emit_card_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("CardContent_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CardContent; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_card_footer(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("CardFooter_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CardFooter; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}
