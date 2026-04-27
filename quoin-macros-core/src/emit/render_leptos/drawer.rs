use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_drawer(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        use super::bindings::next_extract_id;
        let open = crate::emit::common::find_arg_expr(el, "open")
            .map(|e| quote! { open={#e} });
        let on_open_change = crate::emit::common::find_arg_expr(el, "on_open_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_open_change={#w} } });
        let direction = crate::emit::common::find_arg_expr(el, "direction")
            .map(|d| quote! { direction={#d} });
        let should_scale_background = crate::emit::common::find_arg_bool(el, "should_scale_background");
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let alias = quote::format_ident!("Drawer_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Drawer; });
        let mut props = TokenStream::new();
        if let Some(o) = open { props.extend(quote! { #o }); }
        if let Some(oc) = on_open_change { props.extend(quote! { #oc }); }
        if let Some(d) = direction { props.extend(quote! { #d }); }
        props.extend(quote! { should_scale_background={#should_scale_background} });
        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

fn make_drawer_component(name: &str, el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
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

pub(crate) fn emit_drawer_trigger(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerTrigger", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerContent", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_overlay(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerOverlay", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_portal(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerPortal", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_header(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerHeader", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_footer(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerFooter", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_title(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerTitle", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_description(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerDescription", el, bindings, inside_for)
}
pub(crate) fn emit_drawer_close(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    make_drawer_component("DrawerClose", el, bindings, inside_for)
}
