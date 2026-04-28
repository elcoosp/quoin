use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node, generic::emit_html_tag_inner, handler::wrap_event_handler};

pub(crate) fn emit_button(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let primary = find_arg_bool(el, "primary");
        let destructive = find_arg_bool(el, "destructive");
        let ghost = find_arg_bool(el, "ghost");
        let disabled = find_arg_bool(el, "disabled");

        let btn_alias = quote::format_ident!("Button_{}", next_extract_id());
        bindings.push(quote! {
            let #btn_alias = leptos_shadcn_ui::Button;
        });

        let variant = if destructive {
            quote! { { leptos_shadcn_ui::ButtonVariant::Destructive } }
        } else if ghost {
            quote! { { leptos_shadcn_ui::ButtonVariant::Ghost } }
        } else if primary {
            quote! { { leptos_shadcn_ui::ButtonVariant::Default } }
        } else {
            quote! { { leptos_shadcn_ui::ButtonVariant::Outline } }
        };

        let on_click_prop: Option<TokenStream> =
            find_arg_expr(el, "on_click").map(|handler_expr| {
                let handler = wrap_event_handler(handler_expr);
                quote! { on_click={ #handler.into() } }
            });

        let class_prop: TokenStream = if let Some(cls) = find_arg_expr(el, "class") {
            quote! { class={ #cls.into() } }
        } else {
            quote! {}
        };

        let mut children = Vec::new();
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }

        let button = if children.is_empty() {
            let props = match on_click_prop {
                Some(oc) => quote! { variant=#variant #class_prop #oc disabled={ #disabled.into() } },
                None => quote! { variant=#variant #class_prop disabled={ #disabled.into() } },
            };
            quote! { <#btn_alias #props /> }
        } else {
            let props = match on_click_prop {
                Some(oc) => quote! { variant=#variant #class_prop #oc disabled={ #disabled.into() } },
                None => quote! { variant=#variant #class_prop disabled={ #disabled.into() } },
            };
            quote! { <#btn_alias #props> #(#children)* </#btn_alias> }
        };

        let wrapped = if let Some(text) = tooltip_text {
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
                            #button
                        </#ttr_alias>
                        <#ttc_alias>
                            {#text}
                        </#ttc_alias>
                    </#tt_alias>
                </#tp_alias>
            }
        } else {
            button
        };

        wrapped
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let inner_button = emit_html_tag_inner(el, "button", bindings, inside_for);

        match tooltip_text {
            Some(text) => quote! {
                <div class={ "relative inline-block group".into() }>
                    #inner_button
                    <div class={ "absolute bottom full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50 hidden group-hover:block".into() }>
                        {#text}
                    </div>
                </div>
            },
            None => inner_button,
        }
    }
}
