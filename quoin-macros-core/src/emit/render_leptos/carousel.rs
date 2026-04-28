use crate::emit::common::{find_arg_string, find_arg_expr};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_carousel(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let orientation = find_arg_expr(el, "orientation")
            .map(|o| quote! { orientation={ #o.into() } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("Carousel_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Carousel; });
        let props = if let Some(o) = orientation { quote! { #o #class_prop } } else { class_prop };
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

fn simple_component(name: &str, el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("{}_{}", name, next_extract_id());
        let comp_ident = quote::format_ident!("{}", name);
        bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_carousel_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CarouselContent", el, bindings, inside_for)
}
pub(crate) fn emit_carousel_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CarouselItem", el, bindings, inside_for)
}
pub(crate) fn emit_carousel_previous(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    button_component("CarouselPrevious", el, bindings, inside_for)
}
pub(crate) fn emit_carousel_next(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    button_component("CarouselNext", el, bindings, inside_for)
}

fn button_component(name: &str, el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let on_click = find_arg_expr(el, "on_click")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_click={ #w.into() } } });
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("{}_{}", name, next_extract_id());
        let comp_ident = quote::format_ident!("{}", name);
        bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
        let props = if let Some(oc) = on_click { quote! { #oc #class_prop } } else { class_prop };
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "button", bindings, inside_for) }
}
