// Imports in this module vary with the `leptos-shadcn`/`dioxus-shadcn` feature:
// some are only used when the shadcn emit path is active. A file-level allow
// keeps every feature combination compiling without per-line noise.
#![allow(unused_imports)]

use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{
    bindings::next_extract_id, emit_node, generic::emit_html_tag_inner, handler::wrap_event_handler,
};

pub(crate) fn emit_button(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let disabled = find_arg_bool(el, "disabled");

        // Canonical `variant:` first (`emit::shadcn_spec`), boolean sugar
        // (`primary:`/`destructive:`/`ghost:`) as the fallback. Unknown
        // values are a hard error.
        let choice = crate::emit::shadcn_spec::canonical_button_variant(el);
        if let Some(error) = choice
            .as_ref()
            .and_then(crate::emit::shadcn_spec::choice_error)
        {
            return error;
        }
        let variant = match choice.and_then(|c| c.ok()) {
            Some("destructive") => quote! { { leptos_shadcn_ui::ButtonVariant::Destructive } },
            Some("ghost") => quote! { { leptos_shadcn_ui::ButtonVariant::Ghost } },
            Some("outline") => quote! { { leptos_shadcn_ui::ButtonVariant::Outline } },
            Some("secondary") => quote! { { leptos_shadcn_ui::ButtonVariant::Secondary } },
            Some("link") => quote! { { leptos_shadcn_ui::ButtonVariant::Link } },
            // `default`/`primary` and the unspecified case share the
            // emphasized default look; the sugar chain below keeps the
            // historical precedence for `disabled:`-only sources.
            Some(_) | None if find_arg_bool(el, "primary") => {
                quote! { { leptos_shadcn_ui::ButtonVariant::Default } }
            }
            Some(_) | None => quote! { { leptos_shadcn_ui::ButtonVariant::Outline } },
        };

        let on_click_prop: Option<TokenStream> =
            find_arg_expr(el, "on_click").map(|handler_expr| {
                let handler = wrap_event_handler(handler_expr);
                quote! { on_click={#handler} }
            });

        let btn_alias = quote::format_ident!("Button_{}", next_extract_id());
        bindings.push(quote! {
            let #btn_alias = leptos_shadcn_ui::Button;
        });

        let class_prop: TokenStream = if let Some(cls) = find_arg_expr(el, "class") {
            quote! { class={#cls} }
        } else {
            quote! {}
        };

        let mut children = Vec::new();
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }

        let button = {
            let props = if let Some(oc) = on_click_prop {
                quote! { variant=#variant #class_prop #oc disabled={#disabled} }
            } else {
                quote! { variant=#variant #class_prop disabled={#disabled} }
            };
            if children.is_empty() {
                quote! { <#btn_alias #props /> }
            } else {
                quote! { <#btn_alias #props> #(#children)* </#btn_alias> }
            }
        };

        if let Some(text) = tooltip_text {
            let tooltip_provider_alias =
                quote::format_ident!("TooltipProvider_{}", next_extract_id());
            let tooltip_alias = quote::format_ident!("Tooltip_{}", next_extract_id());
            let tooltip_trigger_alias =
                quote::format_ident!("TooltipTrigger_{}", next_extract_id());
            let tooltip_content_alias =
                quote::format_ident!("TooltipContent_{}", next_extract_id());

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
                            #button
                        </#tooltip_trigger_alias>
                        <#tooltip_content_alias>
                            {#text}
                        </#tooltip_content_alias>
                    </#tooltip_alias>
                </#tooltip_provider_alias>
            }
        } else {
            button
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let inner_button = emit_html_tag_inner(el, "button", bindings, inside_for);

        match tooltip_text {
            Some(text) => quote! {
                <div class="relative inline-block group">
                    #inner_button
                    <div class="absolute bottom full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50 hidden group-hover:block">
                        {#text}
                    </div>
                </div>
            },
            None => inner_button,
        }
    }
}
