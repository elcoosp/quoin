use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
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
                    items.push(quote! { value: {#value.get()} });
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

    // Each td cell must wrap the closure call in braces so rsx! parses it
    // as an expression child, not as an attribute or element name.
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
                    // Wrap in extra braces: td { { (closure)(row) } }
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
