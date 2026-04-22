use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{
    collect_handler_idents, collect_handler_idents_excluding_params, force_move_on_closure,
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    quote! {
        {
            use dioxus::prelude::dioxus_elements;
            let __quoin_node: dioxus::prelude::Element = dioxus::prelude::rsx! { #inner };
            __quoin_node
        }
    }
}

fn wrap_with_cfg(attrs: &[syn::Attribute], inner: TokenStream) -> TokenStream {
    let cfg_attrs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    if cfg_attrs.is_empty() {
        inner
    } else {
        quote! { { #(#cfg_attrs)* { #inner } } }
    }
}

fn emit_render_inner(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { {#e} },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
            quote! { #(#tokens)* }
        }
    }
}

fn wrap_dioxus_handler(handler_expr: &syn::Expr) -> TokenStream {
    let idents = collect_handler_idents_excluding_params(handler_expr);
    let shadows: Vec<TokenStream> = idents
        .iter()
        .map(|id| quote! { let #id = #id.clone(); })
        .collect();
    let handler_with_move = force_move_on_closure(handler_expr);
    quote! {
        {
            #(#shadows)*
            #handler_with_move
        }
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let inner = emit_element_inner(el);
    wrap_with_cfg(&el.attrs, inner)
}

fn emit_element_inner(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let effective_name = match name_str.as_str() {
        "tab_bar" => "tabs",
        other => other,
    };

    match effective_name {
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el),
        "dropdown_menu" => emit_dropdown_menu(el),
        "virtual_list" => {
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            quote! { div { style: "overflow-y: auto", #(#children_tokens)* } }
        }
        "clipboard_button" => emit_html_el(el, "button"),
        _ => emit_html_el(el, &name_str),
    }
}

fn emit_html_el(el: &Element, name_str: &str) -> TokenStream {
    let tag = match name_str {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        "button" => "button",
        "input" => "input",
        "span" => "span",
        "label" => "label",
        "a" => "a",
        "ul" => "ul",
        "ol" => "ol",
        "li" => "li",
        "hr" => "hr",
        "br" => "br",
        "textarea" => "textarea",
        "select" => "select",
        "form" => "form",
        "img" => "img",
        _ => "div",
    };

    let has_value = el.args.iter().any(|a| a.key == "value");
    let has_on_input = el.args.iter().any(|a| a.key == "on_input");
    let auto_bind_input = tag == "input" && has_value && !has_on_input;

    let mut items = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "on_click" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onclick: #handler })
            }
            "on_mouse_down" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onmousedown: #handler })
            }
            "on_mouse_up" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onmouseup: #handler })
            }
            "on_mouse_enter" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onmouseenter: #handler })
            }
            "on_mouse_leave" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onmouseleave: #handler })
            }
            "on_input" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { oninput: #handler })
            }
            "on_change" => {
                let handler = wrap_dioxus_handler(value);
                items.push(quote! { onchange: #handler })
            }
            "value" => {
                if tag == "input" {
                    items.push(quote! { value: "{#value.get()}" });
                } else {
                    items.push(quote! { value: {#value} });
                }
            }
            "primary" | "ghost" | "destructive" | "active" | "children" | "trigger" | "rows"
            | "striped" | "items" | "estimated_height" | "copy_text" | "sortable" | "width"
            | "resizable" | "selectable" | "on_sort" | "bordered" | "size" | "navigate_to"
            | "cfg" | "label" | "render" | "key" | "index" => {}
            _ => {
                let key = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
                items.push(quote! { #key: {#value} });
            }
        }
    }

    if auto_bind_input {
        let value_expr = el.args.iter().find(|a| a.key == "value").map(|a| &a.value).unwrap();
        items.push(quote! {
            oninput: move |ev: dioxus::prelude::Event<web_sys::InputEvent>| {
                #value_expr.set(ev.value());
            }
        });
    }

    if let Some(children_expr) = &el.children_expr {
        items.push(quote! { {#children_expr.into_iter()} });
    }
    for child in &el.children {
        items.push(emit_render_inner(child));
    }
    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if items.is_empty() {
        quote! { #tag_ident {} }
    } else {
        quote! { #tag_ident { #(#items),* } }
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let inner = emit_if_inner(if_node);
    wrap_with_cfg(&if_node.attrs, inner)
}

fn emit_if_inner(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens = emit_nodes_inner(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = emit_nodes_inner(else_branch);
        quote! { if #cond { #then_tokens } else { #else_tokens } }
    } else {
        quote! { if #cond { #then_tokens } }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
    let inner = emit_for_inner(for_node);
    wrap_with_cfg(&for_node.attrs, inner)
}

fn emit_for_inner(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes_inner(&for_node.body);
    quote! { for #pat in #iterable { #body } }
}

fn emit_nodes_inner(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render_inner).collect();
    quote! { #(#tokens)* }
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

fn emit_tabs(el: &Element) -> TokenStream {
    let active_expr = el
        .args
        .iter()
        .find(|a| a.key == "active")
        .map(|a| &a.value)
        .expect("tabs require 'active' argument");
    let on_click_expr = el
        .args
        .iter()
        .find(|a| a.key == "on_click")
        .map(|a| &a.value)
        .expect("tabs require 'on_click' callback");

    let param_idents: Vec<proc_macro2::Ident> = if let syn::Expr::Closure(closure) = on_click_expr {
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

    let param_names: std::collections::HashSet<String> =
        param_idents.iter().map(|id| id.to_string()).collect();

    let body_idents: Vec<proc_macro2::Ident> = collect_handler_idents(on_click_expr)
        .into_iter()
        .filter(|id| !param_names.contains(&id.to_string()))
        .collect();

    let on_click_with_move = force_move_on_closure(on_click_expr);

    let per_tab: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "tab" {
                    let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                    let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;

                    let param_shadows: Vec<TokenStream> = param_idents
                        .iter()
                        .map(|id| quote! { let #id = #index; })
                        .collect();
                    let clone_shadows: Vec<TokenStream> = body_idents
                        .iter()
                        .map(|id| quote! { let #id = #id.clone(); })
                        .collect();
                    let call_args: Vec<TokenStream> =
                        param_idents.iter().map(|id| quote! { #id }).collect();

                    return Some(quote! {
                        div {
                            class: if #index == __active { "px-4 py-2 cursor-pointer text-white" } else { "px-4 py-2 cursor-pointer text-gray-400" },
                            onclick: {
                                #(#param_shadows)*
                                #(#clone_shadows)*
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#(#call_args)*) }
                            },
                            #label
                        }
                    });
                }
            }
            None
        })
        .collect();

    quote! {
        {
            let __active = #active_expr;
            dioxus::prelude::rsx! {
                div { class: "flex",
                    #(#per_tab)*
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dropdown menu — use_signal(bool) for open state + conditional rendering
// ---------------------------------------------------------------------------

fn emit_dropdown_menu(el: &Element) -> TokenStream {
    let trigger_expr = match &el.trigger_expr {
        Some(e) => e,
        None => return quote! { div { "dropdown: missing trigger" } },
    };

    // Build item elements from `item(label: ..., on_click: ...)` children
    let item_tokens: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "item" {
                    let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                    let on_click = e.args.iter().find(|a| a.key == "on_click").map(|a| &a.value)?;

                    let handler = wrap_dioxus_handler(on_click);
                    Some(quote! {
                        div {
                            class: "px-3 py-2 cursor-pointer text-white hover:bg-gray-600",
                            onclick: {
                                let __item_handler = #handler;
                                move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                                    ev.stop_propagation();
                                    __open.set(false);
                                    __item_handler(ev);
                                }
                            },
                            #label
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Emit the trigger as an inline expression child
    let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone()));

    quote! {
        {
            let mut __open = dioxus::prelude::use_signal(|| false);
            dioxus::prelude::rsx! {
                div {
                    class: "relative inline-block",
                    onclick: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                        ev.stop_propagation();
                        __open.toggle();
                    },
                    #trigger_inner,
                    if *__open.read() {
                        div {
                            class: "absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg",
                            onclick: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                                ev.stop_propagation();
                            },
                            onmousedown: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                                ev.prevent_default();
                            },
                            #(#item_tokens)*
                        }
                    }
                }
            }
        }
    }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows = el
        .args
        .iter()
        .find(|a| a.key == "rows")
        .map(|a| &a.value)
        .unwrap();

    let header_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let label = e
                        .args
                        .iter()
                        .find(|a| a.key == "label")
                        .map(|a| &a.value)
                        .unwrap();
                    return Some(quote!(th { #label }));
                }
            }
            None
        })
        .collect();

    let row_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let render_closure = e
                        .args
                        .iter()
                        .find(|a| a.key == "render")
                        .map(|a| &a.value)
                        .unwrap();
                    return Some(quote!(td { { (#render_closure)(&__row) } }));
                }
            }
            None
        })
        .collect();

    quote!(
        table {
            thead { tr { #(#header_cells)* } }
            tbody {
                for __row in #rows {
                    tr { #(#row_cells)* }
                }
            }
        }
    )
}
