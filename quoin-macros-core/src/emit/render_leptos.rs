use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    let view_block = quote! { { use leptos::prelude::*; view! { #inner } } };
    wrap_with_cfg(node.attrs(), view_block)
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
        RenderNode::Expr(e) => quote! { { #e } },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
            quote! { #(#tokens)* }
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
        "dropdown_menu" => {
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            quote! { <div> #(#children_tokens)* </div> }
        }
        "virtual_list" => {
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            quote! { <div style="overflow-y: auto"> #(#children_tokens)* </div> }
        }
        "clipboard_button" => emit_html_tag(el, "button"),
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el),
        _ => emit_html_tag(
            el,
            match effective_name {
                "div" => "div",
                "h1" => "h1",
                "h2" => "h2",
                "h3" => "h3",
                "p" | "text" => "p",
                "button" => "button",
                _ => "div",
            },
        ),
    }
}

fn emit_html_tag(el: &Element, tag: &str) -> TokenStream {
    let mut attrs = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "placeholder" => attrs.push(quote! { placeholder=#value }),
            "value" => attrs.push(quote! { value=#value }),
            "disabled" => attrs.push(quote! { disabled=#value }),
            "on_click" => attrs.push(quote! { on:click=#value }),
            "on_mouse_down" => attrs.push(quote! { on:mousedown=#value }),
            "on_input" => attrs.push(quote! { on:input=#value }),
            "on_change" => attrs.push(quote! { on:change=#value }),
            _ => {}
        }
    }
    let mut children = Vec::new();
    if let Some(children_expr) = &el.children_expr {
        children.push(quote! { {#children_expr} });
    } else {
        for child in &el.children {
            children.push(emit_render_inner(child));
        }
    }
    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if children.is_empty() {
        quote! { <#tag_ident #(#attrs)* /> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children)* </#tag_ident> }
    }
}

// AFTER (fixed — .expect() runs at macro-expansion time, HTML-like syntax)
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

    let tab_labels: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "tab" {
                    let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value);
                    let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value);
                    if let (Some(label), Some(index)) = (label, index) {
                        return Some(quote! {
                            <li
                                class={move || if *#index == #active_expr { "active" } else { "" }}
                                on:click={move |_| (#on_click_expr)(*#index)}
                            >#label</li>
                        });
                    }
                }
            }
            None
        })
        .collect();
    quote! { <ul class="tabs"> #(#tab_labels)* </ul> }
}
fn emit_data_table(el: &Element) -> TokenStream {
    let rows = el.args.iter().find(|a| a.key == "rows").map(|a| &a.value);
    let on_sort = el
        .args
        .iter()
        .find(|a| a.key == "on_sort")
        .map(|a| &a.value);
    let striped = find_arg_bool(el, "striped");

    let header_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value);
                    let key = e.args.iter().find(|a| a.key == "key").map(|a| &a.value);
                    let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);

                    // Default empty string expression with a let binding to extend lifetime.
                    let empty_str: syn::Expr = syn::parse_quote! { "" };
                    let label_expr = label.unwrap_or(&empty_str);
                    let key_str = key
                        .and_then(|k| {
                            if let syn::Expr::Lit(lit) = k {
                                if let syn::Lit::Str(s) = &lit.lit {
                                    Some(s.value())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    let mut attrs = vec![quote! { class="px-3 py-2 text-gray-400 font-medium" }];
                    if let Some(w) = width {
                        attrs.push(quote! { style=format!("width: {}px", #w) });
                    }

                    if find_arg_bool(e, "sortable") {
                        if let Some(on_sort_expr) = on_sort {
                            let on_click = quote! { move |_| { #on_sort_expr(#key_str, "asc"); } };
                            attrs.push(quote! { on:click=#on_click });
                            attrs[0] = quote! { class="px-3 py-2 text-gray-400 font-medium cursor-pointer hover:bg-gray-700" };
                        } else {
                            attrs.push(quote! { class="px-3 py-2 text-gray-400 font-medium cursor-pointer" });
                        }
                    }

                    return Some(quote! { <th #(#attrs)*> #label_expr </th> });
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
                    let render_closure =
                        e.args.iter().find(|a| a.key == "render").map(|a| &a.value);
                    let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);
                    if let Some(render_closure) = render_closure {
                        let mut attrs = vec![quote! { class="px-3 py-2 text-white" }];
                        if let Some(w) = width {
                            attrs.push(quote! { style=format!("width: {}px", #w) });
                        }
                        return Some(quote! { <td #(#attrs)*> {#render_closure}(&__row) </td> });
                    }
                }
            }
            None
        })
        .collect();

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows_expr = rows.unwrap_or(&empty_rows);
    let striped_class = if striped { " table-striped" } else { "" };
    quote! {
        <table class={concat!("w-full", #striped_class)}>
            <thead><tr> #(#header_cells)* </tr></thead>
            <tbody>
                {#rows_expr.iter().map(|__row| view! { <tr> #(#row_cells)* </tr> }).collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let inner = emit_if_inner(if_node);
    wrap_with_cfg(&if_node.attrs, inner)
}

fn emit_if_inner(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);

    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = if else_branch.len() == 1 {
            match &else_branch[0] {
                RenderNode::If(nested_if) => {
                    // Nested if: get the closure, then call it from the outer else
                    emit_if_inner(nested_if)
                }
                _ => {
                    let else_nodes = emit_nodes(else_branch);
                    quote! { move || ::leptos::prelude::view! { #else_nodes }.into_any() }
                }
            }
        } else {
            let else_nodes = emit_nodes(else_branch);
            quote! { move || ::leptos::prelude::view! { #else_nodes }.into_any() }
        };

        quote! {
            {
                move || if #cond {
                    ::leptos::prelude::view! { #then_branch }.into_any()
                } else {
                    (#else_tokens)()
                }
            }
        }
    } else {
        quote! {
            {
                move || #cond.then(|| {
                    ::leptos::prelude::view! { #then_branch }.into_any()
                })
            }
        }
    }
}
fn emit_for(for_node: &ForNode) -> TokenStream {
    let inner = emit_for_inner(for_node);
    wrap_with_cfg(&for_node.attrs, inner)
}

fn emit_for_inner(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes(&for_node.body);
    quote! {
        <leptos::prelude::For
            each=move || #iterable.clone().into_iter().collect::<Vec<_>>()
            key=|item| item.id
            children=move |#pat| view! { #body }
        />
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let tokens: Vec<_> = nodes.iter().map(emit_render_inner).collect();
    quote! { #(#tokens)* }
}

fn find_arg_bool(el: &Element, key: &str) -> bool {
    el.args
        .iter()
        .find(|a| a.key == key)
        .map(|a| {
            if let syn::Expr::Lit(expr_lit) = &a.value {
                if let syn::Lit::Bool(b) = &expr_lit.lit {
                    return b.value;
                }
            }
            false
        })
        .unwrap_or(false)
}
