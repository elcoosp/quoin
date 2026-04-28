use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use crate::transpile::force_move_on_closure;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, handler::wrap_event_handler};

pub(crate) fn emit_tabs(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let on_click_wrapped = wrap_event_handler(on_click_expr);

        let tabs_alias = quote::format_ident!("Tabs_{}", next_extract_id());
        let tabs_list_alias = quote::format_ident!("TabsList_{}", next_extract_id());
        let tabs_trigger_alias = quote::format_ident!("TabsTrigger_{}", next_extract_id());

        bindings.push(quote! {
            let #tabs_alias = leptos_shadcn_ui::Tabs;
            let #tabs_list_alias = leptos_shadcn_ui::TabsList;
            let #tabs_trigger_alias = leptos_shadcn_ui::TabsTrigger;
        });

        let tab_triggers: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let tab_label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;
                    Some(quote! {
                        <#tabs_trigger_alias value={#index.to_string()} class={ "text-white".into() }>{#tab_label}</#tabs_trigger_alias>
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            <#tabs_alias
                value={leptos::prelude::Signal::derive(move || (#active_expr).to_string()).into()}
                on_value_change={
                    let __on_click = #on_click_wrapped;
                    move |val: String| {
                        if let Ok(idx) = val.parse::<usize>() {
                            __on_click(idx);
                        }
                    }
                }
            >
                <#tabs_list_alias>
                    #(#tab_triggers)*
                </#tabs_list_alias>
            </#tabs_alias>
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let param_idents: Vec<proc_macro2::Ident> =
            if let syn::Expr::Closure(closure) = on_click_expr {
                closure
                    .inputs
                    .iter()
                    .filter_map(|pat| {
                        if let syn::Pat::Ident(pat_ident) = pat {
                            Some(pat_ident.ident.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

        let on_click_wrapped = wrap_event_handler(on_click_expr);

        let tab_labels: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let tab_label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;

                    let param_shadows: Vec<TokenStream> = param_idents
                        .iter()
                        .map(|id| quote! { let #id = #index; })
                        .collect();
                    let call_args: Vec<TokenStream> =
                        param_idents.iter().map(|id| quote! { #id }).collect();

                    Some(quote! {
                        <li
                            class={move || if #index == #active_expr { "active" } else { "" }}
                            on:click={
                                #(#param_shadows)*
                                let __tab_on_click = #on_click_wrapped;
                                move |_| { __tab_on_click(#(#call_args)*) }
                            }
                        >{#tab_label}</li>
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! { <ul class={ "tabs".into() }> #(#tab_labels)* </ul> }
    }
}
