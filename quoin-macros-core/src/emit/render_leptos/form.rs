use crate::emit::common::{find_arg_expr, find_arg_string, find_arg_bool};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_form(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let on_submit = find_arg_expr(el, "on_submit")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_submit={#w} } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("Form_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Form; });
        let props = if let Some(oc) = on_submit { quote! { #oc #class_prop } } else { class_prop };
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "form", bindings, inside_for) }
}

pub(crate) fn emit_form_field(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let name = find_arg_expr(el, "name")
            .map(|e| quote! { name={#e} });
        let invalid = find_arg_bool(el, "invalid");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("FormField_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::FormField; });
        let props = if let Some(n) = name { quote! { #n invalid={#invalid} } } else { quote! { invalid={#invalid} } };
        if children.is_empty() { quote! { <#alias #props #class_prop /> } } else { quote! { <#alias #props #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

fn simple_component(name: &str, el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("{}_{}", name, next_extract_id());
        let comp_ident = quote::format_ident!("{}", name);
        bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { super::generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_form_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("FormItem", el, bindings, inside_for)
}
pub(crate) fn emit_form_label(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let for_field = find_arg_expr(el, "for_field")
            .map(|e| quote! { for_field={#e} });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("FormLabel_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::FormLabel; });
        let props = if let Some(f) = for_field { quote! { #f #class_prop } } else { class_prop };
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "label", bindings, inside_for) }
}
pub(crate) fn emit_form_control(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("FormControl", el, bindings, inside_for)
}
pub(crate) fn emit_form_message(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let message = find_arg_expr(el, "message")
            .map(|e| quote! { message={#e} });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("FormMessage_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::FormMessage; });
        let props = if let Some(m) = message { quote! { #m #class_prop } } else { class_prop };
        quote! { <#alias #props /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
pub(crate) fn emit_form_description(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("FormDescription", el, bindings, inside_for)
}
