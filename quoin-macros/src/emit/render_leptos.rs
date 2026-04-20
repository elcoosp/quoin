// quoin-macros/src/emit/render_leptos.rs
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    quote! { { use leptos::prelude::*; view! { #inner } } }
}

fn emit_render_inner(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { { #e } },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "button" => emit_button(el),
        "input" => emit_input(el),
        _ => emit_builtin(el),
    }
}

fn emit_button(el: &Element) -> TokenStream {
    let mut attrs = Vec::new();

    for (key, value) in &el.args {
        let key_str = key.to_string();
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "on_click" => attrs.push(quote! { on:click=#value }),
            _ => {}
        }
    }

    // Determine variant class
    let primary = find_arg_bool(el, "primary");
    let ghost = find_arg_bool(el, "ghost");
    let destructive = find_arg_bool(el, "destructive");
    let variant_class = if primary {
        "bg-blue-600 text-white hover:bg-blue-700"
    } else if destructive {
        "bg-red-600 text-white hover:bg-red-700"
    } else if ghost {
        "bg-transparent text-white hover:bg-gray-800"
    } else {
        "bg-gray-600 text-white hover:bg-gray-700"
    };

    let mut children = Vec::new();
    for child in &el.children {
        children.push(emit_render_inner(child));
    }

    let tag_ident = proc_macro2::Ident::new("button", proc_macro2::Span::call_site());
    if children.is_empty() {
        quote! { <#tag_ident class=#variant_class #(#attrs)* /> }
    } else {
        quote! { <#tag_ident class=#variant_class #(#attrs)*> #(#children)* </#tag_ident> }
    }
}

fn emit_input(el: &Element) -> TokenStream {
    let mut attrs = Vec::new();

    for (key, value) in &el.args {
        let key_str = key.to_string();
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "placeholder" => attrs.push(quote! { placeholder=#value }),
            "value" => attrs.push(quote! { value=#value }),
            "on_input" => attrs.push(quote! { on:input=#value }),
            _ => {}
        }
    }

    quote! { <input type="text" #(#attrs)* /> }
}

fn emit_builtin(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let tag = match name_str.as_str() {
        "div" => "div",
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" => "p",
        _ => "div",
    };
    let mut attrs = Vec::new();
    for (key, value) in &el.args {
        let key_str = key.to_string();
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "on_click" => attrs.push(quote! { on:click=#value }),
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

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        quote! { {move || if #cond { #then_branch } else { #else_branch }} }
    } else {
        quote! { {move || if #cond { #then_branch }} }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
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
        .find(|(k, _)| k == key)
        .map(|(_, v)| {
            if let syn::Expr::Lit(expr_lit) = v {
                if let syn::Lit::Bool(b) = &expr_lit.lit {
                    return b.value;
                }
            }
            false
        })
        .unwrap_or(false)
}
