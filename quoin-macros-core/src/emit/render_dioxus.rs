use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
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
        "virtual_list" => {
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            quote! { div { style: "overflow-y: auto", #(#children_tokens)* } }
        }
        "dropdown_menu" => {
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            quote! { div { #(#children_tokens)* } }
        }
        "clipboard_button" => emit_builtin(el, "button"),
        _ => emit_builtin(el, &name_str),
    }
}

fn emit_builtin(el: &Element, name_str: &str) -> TokenStream {
    let tag = match name_str {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        "button" => "button",
        "input" => "input",
        _ => "div",
    };
    let mut items = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "on_click" => items.push(quote! { onclick: #value }),
            "on_mouse_down" => items.push(quote! { onmousedown: #value }),
            "on_input" => items.push(quote! { oninput: #value }),
            "on_change" => items.push(quote! { onchange: #value }),
            "class" => items.push(quote! { class: #value }),
            "id" => items.push(quote! { id: #value }),
            "placeholder" => items.push(quote! { placeholder: #value }),
            "value" => items.push(quote! { value: #value }),
            "disabled" => items.push(quote! { disabled: #value }),
            _ => {
                let key = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
                items.push(quote! { #key: #value });
            }
        }
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

fn emit_tabs(_el: &Element) -> TokenStream {
    quote! { div {} }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows = el
        .args
        .iter()
        .find(|a| a.key == "rows")
        .map(|a| &a.value)
        .unwrap();
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
                    let label = e
                        .args
                        .iter()
                        .find(|a| a.key == "label")
                        .map(|a| &a.value)
                        .unwrap();
                    let key = e.args.iter().find(|a| a.key == "key").map(|a| &a.value);
                    let sortable = find_arg_bool(e, "sortable");
                    let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);

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

                    let mut attrs = vec![quote!(class: "px-3 py-2 text-gray-400 font-medium")];
                    if let Some(w) = width {
                        attrs.push(quote!(style: format!("width: {}px", #w)));
                    }

                    if sortable {
                        if let Some(on_sort_expr) = on_sort {
                            let on_click = quote!(move |_| { #on_sort_expr(#key_str, "asc"); });
                            attrs.push(quote!(onclick: #on_click));
                            attrs[0] =
                                quote!(class: "px-3 py-2 text-gray-400 font-medium cursor-pointer");
                        } else {
                            attrs.push(
                                quote!(class: "px-3 py-2 text-gray-400 font-medium cursor-pointer"),
                            );
                        }
                    }

                    return Some(quote!(th { #(#attrs)* } #label));
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
                    let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);
                    let mut attrs = vec![quote!(class: "px-3 py-2 text-white")];
                    if let Some(w) = width {
                        attrs.push(quote!(style: format!("width: {}px", #w)));
                    }
                    return Some(quote!(td { #(#attrs)* } (#render_closure)(&__row)));
                }
            }
            None
        })
        .collect();

    let striped_attr = if striped {
        quote!(striped: true)
    } else {
        quote!()
    };
    quote!(
        table { #striped_attr
            thead { tr { #(#header_cells)* } }
            tbody { #rows.iter().map(|__row| { rsx! { tr { #(#row_cells)* } } }) }
        }
    )
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
    quote! { {#iterable.into_iter().map(|#pat| #body)} }
}

fn emit_nodes_inner(nodes: &[RenderNode]) -> TokenStream {
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
