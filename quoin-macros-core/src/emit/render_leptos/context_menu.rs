use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_context_menu(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let open = find_arg_expr(el, "open")
            .map(|e| quote! { open={#e} });
        let on_open_change = find_arg_expr(el, "on_open_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_open_change={#w} } });

        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let alias = quote::format_ident!("ContextMenu_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::ContextMenu; });

        let props = if let (Some(o), Some(oc)) = (open.clone(), on_open_change.clone()) {
            quote! { #o #oc }
        } else if let Some(o) = open {
            quote! { #o }
        } else { quote! {} };

        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_context_menu_trigger(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuTrigger", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuContent", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuItem", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_separator(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuSeparator", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_label(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuLabel", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_checkbox_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let checked = find_arg_expr(el, "checked")
            .map(|e| quote! { checked={#e} });
        let on_checked_change = find_arg_expr(el, "on_checked_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_checked_change={#w} } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("ContextMenuCheckboxItem_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::ContextMenuCheckboxItem; });
        let props = if let (Some(c), Some(oc)) = (checked.clone(), on_checked_change.clone()) {
            quote! { #c #oc }
        } else if let Some(c) = checked.clone() { quote! { #c } } else { quote! {} };
        if children.is_empty() { quote! { <#alias #props #class_prop /> } } else { quote! { <#alias #props #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
pub(crate) fn emit_context_menu_radio_group(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={#e} });
        let on_value_change = find_arg_expr(el, "on_value_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_value_change={#w} } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("ContextMenuRadioGroup_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::ContextMenuRadioGroup; });
        let props = if let (Some(v), Some(oc)) = (value.clone(), on_value_change.clone()) {
            quote! { #v #oc }
        } else if let Some(v) = value.clone() { quote! { #v } } else { quote! {} };
        if children.is_empty() { quote! { <#alias #props #class_prop /> } } else { quote! { <#alias #props #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
pub(crate) fn emit_context_menu_radio_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={#e} });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("ContextMenuRadioItem_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::ContextMenuRadioItem; });
        if children.is_empty() { quote! { <#alias #value #class_prop /> } } else { quote! { <#alias #value #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
pub(crate) fn emit_context_menu_sub(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuSub", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_sub_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuSubContent", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_sub_trigger(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuSubTrigger", el, bindings, inside_for)
}
pub(crate) fn emit_context_menu_shortcut(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("ContextMenuShortcut", el, bindings, inside_for)
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
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}
