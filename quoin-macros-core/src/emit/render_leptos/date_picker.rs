use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use super::{bindings::next_extract_id, generic};

pub(crate) fn emit_date_picker(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let selected = find_arg_expr(el, "selected")
            .map(|s| quote! { selected={#s} })
            .unwrap_or_else(|| quote! {});
        let on_select = find_arg_expr(el, "on_select")
            .map(|h| { let w = super::handler::wrap_event_handler(h); quote! { on_select={#w} } })
            .unwrap_or_else(|| quote! {});
        let disabled = crate::emit::common::find_arg_expr(el, "disabled")
            .map(|d| quote! { disabled={#d} })
            .unwrap_or_else(|| quote! {});
        let placeholder = crate::emit::common::find_arg_string(el, "placeholder")
            .map(|p| quote! { placeholder={#p} })
            .unwrap_or_else(|| quote! {});
        let class = crate::emit::common::find_arg_string(el, "class").unwrap_or_default();
        let class_prop = if class.is_empty() { quote! {} } else { quote! { class={#class} } };
        let alias = quote::format_ident!("DatePicker_{}", next_extract_id());
        bindings.push(quote! { let #alias = leptos_shadcn_ui::DatePicker; });
        quote! { <#alias #selected #on_select #disabled #placeholder #class_prop /> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    { generic::emit_html_tag(el, "input", bindings, _inside_for) }
}
