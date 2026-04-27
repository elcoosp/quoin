use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_alert_dialog(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        use super::bindings::next_extract_id;
        let open = crate::emit::common::find_arg_expr(el, "open")
            .map(|e| quote! { open={#e} });
        let on_open_change = crate::emit::common::find_arg_expr(el, "on_open_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_open_change={#w} } });
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let alias = quote::format_ident!("AlertDialog_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AlertDialog; });
        let props = if let (Some(o), Some(oc)) = (open.clone(), on_open_change.clone()) { quote! { #o #oc } }
                    else if let Some(o) = open.clone() { quote! { #o } }
                    else { quote! {} };
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

fn make_alert_component(name: &str, el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        use super::bindings::next_extract_id;
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
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

pub(crate) fn emit_alert_dialog_trigger(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogTrigger", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_overlay(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogOverlay", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_header(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogHeader", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_footer(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogFooter", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_title(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogTitle", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_description(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogDescription", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_action(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogAction", el, bindings, inside_for)
}
pub(crate) fn emit_alert_dialog_cancel(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_alert_component("AlertDialogCancel", el, bindings, inside_for)
}
