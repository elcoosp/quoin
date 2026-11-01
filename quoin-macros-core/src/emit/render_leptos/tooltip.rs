// Imports in this module vary with the `leptos-shadcn`/`dioxus-shadcn` feature:
// some are only used when the shadcn emit path is active. A file-level allow
// keeps every feature combination compiling without per-line noise.
#![allow(unused_imports)]

use crate::emit::common::find_arg_string;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node};

pub(crate) fn emit_tooltip(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let trigger_expr = &el.trigger_expr;
    let text = find_arg_string(el, "text").unwrap_or_default();

    if trigger_expr.is_none() {
        return quote! { <span title={#text}>{#text}</span> };
    }

    let trigger_inner = emit_node(
        &RenderNode::Expr(trigger_expr.clone().unwrap()),
        bindings,
        inside_for,
    );

    #[cfg(feature = "leptos-shadcn")]
    {
        let tooltip_provider_alias = quote::format_ident!("TooltipProvider_{}", next_extract_id());
        let tooltip_alias = quote::format_ident!("Tooltip_{}", next_extract_id());
        let tooltip_trigger_alias = quote::format_ident!("TooltipTrigger_{}", next_extract_id());
        let tooltip_content_alias = quote::format_ident!("TooltipContent_{}", next_extract_id());

        bindings.push(quote! {
            let #tooltip_provider_alias = leptos_shadcn_ui::TooltipProvider;
            let #tooltip_alias = leptos_shadcn_ui::Tooltip;
            let #tooltip_trigger_alias = leptos_shadcn_ui::TooltipTrigger;
            let #tooltip_content_alias = leptos_shadcn_ui::TooltipContent;
        });
        quote! {
            <#tooltip_provider_alias>
                <#tooltip_alias>
                    <#tooltip_trigger_alias>
                        {#trigger_inner}
                    </#tooltip_trigger_alias>
                    <#tooltip_content_alias>
                        {#text}
                    </#tooltip_content_alias>
                </#tooltip_alias>
            </#tooltip_provider_alias>
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        quote! {
            <div class="relative inline-block group">
                {#trigger_inner}
                <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50 hidden group-hover:block">
                    {#text}
                </div>
            </div>
        }
    }
}
