use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic, handler::wrap_event_handler};

pub(crate) fn emit_pagination(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let current_page = find_arg_expr(el, "current_page")
            .map(|e| quote! { current_page={leptos::prelude::Signal::derive(move || #e)} })
            .unwrap_or_else(|| quote! {});
        let total_pages = find_arg_expr(el, "total_pages")
            .map(|e| quote! { total_pages={#e} })
            .unwrap_or_else(|| quote! { total_pages=1usize });
        let on_page_change = find_arg_expr(el, "on_page_change")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_page_change={#w} } })
            .unwrap_or_else(|| quote! {});
        let show_previous_next = find_arg_bool(el, "show_previous_next");
        let show_first_last = find_arg_bool(el, "show_first_last");
        let class = find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };

        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let alias = quote::format_ident!("Pagination_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Pagination; });

        if children.is_empty() {
            quote! { <#alias #current_page #total_pages #on_page_change show_previous_next={#show_previous_next} show_first_last={#show_first_last} #class_prop /> }
        } else {
            quote! { <#alias #current_page #total_pages #on_page_change show_previous_next={#show_previous_next} show_first_last={#show_first_last} #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "nav", bindings, inside_for) }
}

pub(crate) fn emit_pagination_content(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationContent_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationContent; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "ul", bindings, inside_for) }
}

pub(crate) fn emit_pagination_item(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationItem_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationItem; });
        if children.is_empty() { quote! { <#alias #class_prop /> } } else { quote! { <#alias #class_prop> #(#children)* </#alias> } }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "li", bindings, inside_for) }
}

pub(crate) fn emit_pagination_link(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let is_active = find_arg_bool(el, "is_active");
        let disabled = find_arg_bool(el, "disabled");
        let on_click = find_arg_expr(el, "on_click")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_click={#w} } })
            .unwrap_or_else(|| quote! {});
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationLink_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationLink; });
        if children.is_empty() {
            quote! { <#alias is_active={#is_active} disabled={#disabled} #on_click #class_prop /> }
        } else {
            quote! { <#alias is_active={#is_active} disabled={#disabled} #on_click #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "a", bindings, inside_for) }
}

pub(crate) fn emit_pagination_previous(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let disabled = find_arg_bool(el, "disabled");
        let on_click = find_arg_expr(el, "on_click")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_click={#w} } })
            .unwrap_or_else(|| quote! {});
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationPrevious_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationPrevious; });
        if children.is_empty() {
            quote! { <#alias disabled={#disabled} #on_click #class_prop /> }
        } else {
            quote! { <#alias disabled={#disabled} #on_click #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "a", bindings, inside_for) }
}

pub(crate) fn emit_pagination_next(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let disabled = find_arg_bool(el, "disabled");
        let on_click = find_arg_expr(el, "on_click")
            .map(|h| { let w = wrap_event_handler(h); quote! { on_click={#w} } })
            .unwrap_or_else(|| quote! {});
        let class = find_arg_string(el, "class").unwrap_or_default();
        let children: Vec<TokenStream> = el.children.iter().map(|c| emit_node(c, bindings, inside_for)).collect();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationNext_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationNext; });
        if children.is_empty() {
            quote! { <#alias disabled={#disabled} #on_click #class_prop /> }
        } else {
            quote! { <#alias disabled={#disabled} #on_click #class_prop> #(#children)* </#alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "a", bindings, inside_for) }
}

pub(crate) fn emit_pagination_ellipsis(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class = find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("PaginationEllipsis_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::PaginationEllipsis; });
        quote! { <#alias #class_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "span", bindings, inside_for) }
}
