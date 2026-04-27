use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node, handler::wrap_event_handler};

pub(crate) fn emit_dropdown_menu(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { <div>dropdown: missing trigger</div> },
        };

        let dm_alias = quote::format_ident!("DropdownMenu_{}", next_extract_id());
        let dmt_alias = quote::format_ident!("DropdownMenuTrigger_{}", next_extract_id());
        let dmc_alias = quote::format_ident!("DropdownMenuContent_{}", next_extract_id());
        let dmi_alias = quote::format_ident!("DropdownMenuItem_{}", next_extract_id());

        bindings.push(quote! {
            let #dm_alias = leptos_shadcn_ui::DropdownMenu;
            let #dmt_alias = leptos_shadcn_ui::DropdownMenuTrigger;
            let #dmc_alias = leptos_shadcn_ui::DropdownMenuContent;
            let #dmi_alias = leptos_shadcn_ui::DropdownMenuItem;
        });

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let item_label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;
                    let handler = wrap_event_handler(on_click);
                    Some(quote! {
                        <#dmi_alias on_click={#handler}>
                            {#item_label}
                        </#dmi_alias>
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_node(
            &RenderNode::Expr(trigger_expr.clone()),
            bindings,
            inside_for,
        );

        quote! {
            <#dm_alias>
                <#dmt_alias>
                    {#trigger_inner}
                </#dmt_alias>
                <#dmc_alias>
                    #(#item_tokens)*
                </#dmc_alias>
            </#dm_alias>
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { <div>dropdown: missing trigger</div> },
        };

        let dd_id = next_extract_id();
        let open_name = quote::format_ident!("__quoin_dd_open_{}", dd_id);
        let node_ref_name = quote::format_ident!("__quoin_dd_ref_{}", dd_id);
        bindings.push(quote! {
            let #open_name = leptos::prelude::create_signal(false);
            let #node_ref_name = leptos::prelude::create_node_ref::<html::Div>();
        });

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let item_label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;
                    let checked = find_arg_bool(e, "checked");
                    let handler = wrap_event_handler(on_click);
                    let close_open = quote! { #open_name.set(false); };
                    let checked_icon = if checked {
                        Some(quote! { <span class="mr-2">"✓"</span> })
                    } else {
                        None
                    };
                    Some(quote! {
                        <div
                            class="px-3 py-2 cursor-pointer text-white hover:bg-gray-600 flex items-center"
                            on:click={
                                let __item_handler = #handler;
                                move |ev: leptos::ev::MouseEvent| {
                                    ev.stop_propagation();
                                    #close_open;
                                    __item_handler(ev);
                                }
                            }
                        >{#checked_icon}{#item_label}</div>
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_node(
            &RenderNode::Expr(trigger_expr.clone()),
            bindings,
            inside_for,
        );

        quote! {
            <div
                node_ref=#node_ref_name
                class="relative inline-block"
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    #open_name.update(|v| *v = !*v);
                }
            >
                {#trigger_inner}
                {
                    move || #open_name.get().then(|| {
                        leptos::view! {
                            <div
                                class="absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg"
                                on:click=move |ev: leptos::ev::MouseEvent| {
                                    ev.stop_propagation();
                                }
                                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                    ev.prevent_default();
                                }
                            >
                                #(#item_tokens)*
                            </div>
                        }.into_any()
                    })
                }
            </div>
        }
    }
}
