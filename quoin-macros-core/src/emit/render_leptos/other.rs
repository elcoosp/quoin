use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

// ---------------------------------------------------------------------------
// Textarea
// ---------------------------------------------------------------------------
pub(crate) fn emit_textarea(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();
        let class_expr = find_arg_expr(el, "class");
        let value_expr = find_arg_expr(el, "value");
        let on_change_expr = find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"));
        let disabled = find_arg_bool(el, "disabled");

        let value_prop = if let Some(val) = value_expr {
            quote! { value={leptos::prelude::Signal::derive(move || #val.get())} }
        } else { quote! {} };

        let on_change_prop = if let Some(handler) = on_change_expr {
            let wrapped = wrap_event_handler(handler);
            quote! { on_change={#wrapped} }
        } else { quote! {} };

        let placeholder_prop = if placeholder.is_empty() { quote! {} } else { quote! { placeholder={#placeholder} } };
        let class_prop = class_expr.map(|c| quote! { class={#c} }).unwrap_or_else(|| quote! {});
        let disabled_prop = if disabled { quote! { disabled=true } } else { quote! {} };

        let alias = quote::format_ident!("Textarea_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Textarea; });
        quote! { <#alias #value_prop #on_change_prop #placeholder_prop #class_prop #disabled_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        generic::emit_html_tag(el, "textarea", bindings, inside_for)
    }
}

// ---------------------------------------------------------------------------
// Task 1.4 stubs: Toggle, HoverCard, Menubar, NavigationMenu, Popover
// ---------------------------------------------------------------------------

pub(crate) fn emit_toggle(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let pressed = find_arg_expr(el, "pressed");
        let on_change = find_arg_expr(el, "on_change");
        let disabled = find_arg_bool(el, "disabled");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();

        let pressed_prop = pressed.map(|p| quote! { pressed={#p} }).unwrap_or_else(|| quote! {});
        let on_change_prop = on_change.map(|h| { let w = wrap_event_handler(h); quote! { on_pressed_change={#w} } }).unwrap_or_else(|| quote! {});
        let disabled_prop = if disabled { quote! { disabled=true } } else { quote! {} };
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };

        let alias = quote::format_ident!("Toggle_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Toggle; });

        if children.is_empty() {
            quote! { <#alias #pressed_prop #on_change_prop #disabled_prop #class_prop /> }
        } else {
            quote! { <#alias #pressed_prop #on_change_prop #disabled_prop #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        generic::emit_html_tag(el, "button", bindings, inside_for)
    }
}



pub(crate) fn emit_menubar(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("Menubar_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Menubar; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

pub(crate) fn emit_navigation_menu(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("NavigationMenu_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::NavigationMenu; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "nav", bindings, inside_for) }
}

pub(crate) fn emit_popover(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("Popover_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Popover; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}

// Stubs for future chunks (prevent compilation errors from mod.rs dispatch)

pub(crate) fn emit_resizable(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    generic::emit_html_tag_inner(el, "div", bindings, inside_for)
}






pub(crate) fn emit_input_otp(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let max_length = crate::emit::common::find_arg_expr(el, "max_length")
            .map(|e| quote! { max_length={#e} })
            .unwrap_or_else(|| quote! { max_length=6usize });
        let value = crate::emit::common::find_arg_expr(el, "value")
            .map(|e| quote! { value={#e} });
        let on_change = crate::emit::common::find_arg_expr(el, "on_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_change={#w} } });
        let on_complete = crate::emit::common::find_arg_expr(el, "on_complete")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_complete={#w} } });
        let disabled = crate::emit::common::find_arg_bool(el, "disabled");
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };

        let alias = quote::format_ident!("InputOtp_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::InputOtp; });

        let mut props = TokenStream::new();
        props.extend(max_length);
        if let Some(v) = value { props.extend(quote! { #v }); }
        if let Some(oc) = on_change { props.extend(quote! { #oc }); }
        if let Some(oc) = on_complete { props.extend(quote! { #oc }); }
        props.extend(quote! { disabled={#disabled} });
        props.extend(class_prop);

        quote! { <#alias #props /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag_inner(el, "div", bindings, inside_for) }
}


