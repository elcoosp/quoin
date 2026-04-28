use crate::emit::common::{find_arg_expr, find_arg_string, find_arg_bool};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_accordion(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let r#type = find_arg_expr(el, "type")
            .map(|e| quote! { type={ #e.into() } });
        let orientation = find_arg_expr(el, "orientation")
            .map(|e| quote! { orientation={ #e.into() } });
        let collapsible = find_arg_expr(el, "collapsible")
            .map(|e| quote! { collapsible={ #e.into() } });
        let disabled = find_arg_bool(el, "disabled");
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={ #e.into() } })
            .expect("Accordion requires 'value' argument (RwSignal<Vec<String>>)");
        let on_value_change = find_arg_expr(el, "on_value_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_value_change={ #w.into() } } });
        let class = find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };

        let mut children: Vec<TokenStream> = Vec::new();
        for child in &el.children {
            if let RenderNode::Element(child_el) = child {
                let child_name = child_el.name.to_string();
                match child_name.as_str() {
                    "accordion_item" => children.push(emit_accordion_item(child_el, bindings, inside_for)),
                    _ => children.push(emit_node(child, bindings, inside_for)),
                }
            } else {
                children.push(emit_node(child, bindings, inside_for));
            }
        }

        let alias = quote::format_ident!("Accordion_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Accordion; });
        let mut props = TokenStream::new();
        if let Some(t) = r#type { props.extend(t); }
        if let Some(o) = orientation { props.extend(o); }
        if let Some(c) = collapsible { props.extend(c); }
        props.extend(quote! { disabled={ #disabled.into() } });
        props.extend(value);
        if let Some(oc) = on_value_change { props.extend(oc); }
        props.extend(class_prop);

        if children.is_empty() { quote! { <#alias #props /> } } else { quote! { <#alias #props> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_accordion_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let value = find_arg_expr(el, "value")
            .map(|e| quote! { value={ #e.into() } })
            .expect("AccordionItem requires 'value'");
        let disabled = find_arg_bool(el, "disabled");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("AccordionItem_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AccordionItem; });
        if children.is_empty() {
            quote! { <#alias #value disabled={ #disabled.into() } #class_prop /> }
        } else {
            quote! { <#alias #value disabled={ #disabled.into() } #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_accordion_trigger(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    simple_component("AccordionTrigger", el, bindings, inside_for)
}

pub(crate) fn emit_accordion_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let force_mount = find_arg_bool(el, "force_mount");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={ #class.into() } } };
        let alias = quote::format_ident!("AccordionContent_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::AccordionContent; });
        if children.is_empty() {
            quote! { <#alias force_mount={ #force_mount.into() } #class_prop /> }
        } else {
            quote! { <#alias force_mount={ #force_mount.into() } #class_prop> #(#children)* </#alias> }
        }
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
