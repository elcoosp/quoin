use crate::emit::common::{find_arg_expr, find_arg_string, find_arg_bool};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_command(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={ #e.into() } });
        let on_value_change = find_arg_expr(el, "on_value_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_value_change={ #w.into() } } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("Command_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Command; });
        let props = if let (Some(v), Some(oc)) = (value.clone(), on_value_change.clone()) {
            quote! { #v #oc }
        } else if let Some(v) = value.clone() { quote! { #v } } else { quote! {} };
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
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("{}_{}", name, next_extract_id());
        let comp_ident = quote::format_ident!("{}", name);
        bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_command_input(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandInput", el, bindings, inside_for)
}
pub(crate) fn emit_command_list(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandList", el, bindings, inside_for)
}
pub(crate) fn emit_command_empty(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandEmpty", el, bindings, inside_for)
}
pub(crate) fn emit_command_group(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandGroup", el, bindings, inside_for)
}
pub(crate) fn emit_command_group_heading(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandGroupHeading", el, bindings, inside_for)
}
pub(crate) fn emit_command_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={ #e.into() } });
        let disabled = find_arg_bool(el, "disabled");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("CommandItem_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::CommandItem; });
        let props = if let Some(v) = value { quote! { #v disabled={ #disabled.into() } } } else { quote! { disabled={ #disabled.into() } } };
        if children.is_empty() { quote! { <#alias #props #class_prop /> } } else { quote! { <#alias #props #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
pub(crate) fn emit_command_shortcut(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandShortcut", el, bindings, inside_for)
}
pub(crate) fn emit_command_separator(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("CommandSeparator", el, bindings, inside_for)
}
