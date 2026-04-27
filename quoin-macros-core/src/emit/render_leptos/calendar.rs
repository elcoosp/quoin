use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, emit_node, generic};

pub(crate) fn emit_calendar(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let mode = find_arg_expr(el, "mode")
            .map(|m| quote! { mode={leptos::prelude::Signal::derive(move || #m)} })
            .unwrap_or_else(|| quote! {});
        let selected = find_arg_expr(el, "selected")
            .map(|s| quote! { selected={#s} })
            .unwrap_or_else(|| quote! {});
        let on_select = find_arg_expr(el, "on_select")
            .map(|h| { let w = super::handler::wrap_event_handler(h); quote! { on_select={#w} } })
            .unwrap_or_else(|| quote! {});
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("Calendar_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::Calendar; });
        quote! { <#alias #mode #selected #on_select #class_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "div", bindings, inside_for) }
}
