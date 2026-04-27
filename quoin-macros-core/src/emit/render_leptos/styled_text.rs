use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::next_extract_id;

pub(crate) fn emit_styled_text(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let text_expr = find_arg_expr(el, "text");
    let query_expr = find_arg_expr(el, "query");

    match (text_expr, query_expr) {
        (Some(text), None) => quote! { <span>{#text}</span> },
        (Some(text), Some(query)) => {
            let hl_id = next_extract_id();
            let hl_name = quote::format_ident!("__quoin_highlight_{}", hl_id);
            bindings.push(quote! {
                let #hl_name = {
                    let __text_val = (#text).clone();
                    let __query_val = (#query).clone();
                    move || {
                        if __query_val.is_empty() {
                            return { use leptos::prelude::*; leptos::view! { <span>{__text_val.clone()}</span> } }.into_any();
                        }
                        let mut __parts: Vec<leptos::prelude::AnyView> = Vec::new();
                        let mut __remaining = __text_val.as_str();
                        let __query_lower = __query_val.to_lowercase();
                        while let Some(__idx) = __remaining.to_lowercase().find(&__query_lower) {
                            if __idx > 0 {
                                let __before: &str = &__remaining[..__idx];
                                __parts.push({ use leptos::prelude::*; leptos::view! { <span>{__before}</span> } }.into_any());
                            }
                            let __match: &str = &__remaining[__idx..__idx + __query_val.len()];
                            __parts.push(
                                { use leptos::prelude::*; leptos::view! { <span class="bg-yellow-200 text-black">{__match}</span> } }.into_any()
                            );
                            __remaining = &__remaining[__idx + __query_val.len()..];
                        }
                        if !__remaining.is_empty() {
                            __parts.push({ use leptos::prelude::*; leptos::view! { <span>{__remaining}</span> } }.into_any());
                        }
                        { use leptos::prelude::*; leptos::view! { <span>{__parts.into_iter()}</span> } }.into_any()
                    }
                };
            });
            quote! { {#hl_name()} }
        }
        (None, _) => quote! { <span></span> },
    }
}
