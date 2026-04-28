use crate::emit::common::find_arg_string;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node};

pub(crate) fn emit_tooltip(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let trigger_expr = &el.trigger_expr;
    let text = find_arg_string(el, "text").unwrap_or_default();

    if trigger_expr.is_none() {
        return quote! { <span title={ #text.into() }>{#text}</span> };
    }

    let trigger_inner = emit_node(
        &RenderNode::Expr(trigger_expr.clone().unwrap()),
        bindings,
        inside_for,
    );

    #[cfg(feature = "leptos-shadcn")]
    {
        let tp_alias = quote::format_ident!("TooltipProvider_{}", next_extract_id());
        let tt_alias = quote::format_ident!("Tooltip_{}", next_extract_id());
        let ttr_alias = quote::format_ident!("TooltipTrigger_{}", next_extract_id());
        let ttc_alias = quote::format_ident!("TooltipContent_{}", next_extract_id());

        bindings.push(quote! {
            let #tp_alias = leptos_shadcn_ui::TooltipProvider;
            let #tt_alias = leptos_shadcn_ui::Tooltip;
            let #ttr_alias = leptos_shadcn_ui::TooltipTrigger;
            let #ttc_alias = leptos_shadcn_ui::TooltipContent;
        });
        quote! {
            <#tp_alias>
                <#tt_alias>
                    <#ttr_alias>
                        {#trigger_inner}
                    </#ttr_alias>
                    <#ttc_alias>
                        {#text}
                    </#ttc_alias>
                </#tt_alias>
            </#tp_alias>
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        quote! {
            <div class={ "relative inline-block group".into() }>
                {#trigger_inner}
                <div class={ "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50 hidden group-hover:block".into() }>
                    {#text}
                </div>
            </div>
        }
    }
}
