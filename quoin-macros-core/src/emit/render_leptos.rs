use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{
    collect_handler_idents, collect_handler_idents_excluding_params, force_move_on_closure,
};
use proc_macro2::TokenStream;
use quote::quote;
use std::sync::atomic::{AtomicUsize, Ordering};

static EXTRACT_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_extract_id() -> usize {
    EXTRACT_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let mut bindings = Vec::new();
    let inner = emit_node(node, &mut bindings, false);

    let tokens = if bindings.is_empty() {
        quote! { { use leptos::prelude::*; view! { #inner } } }
    } else {
        quote! { { use leptos::prelude::*; #(#bindings)* view! { #inner } } }
    };

    wrap_with_cfg(node.attrs(), tokens)
}

// ---------------------------------------------------------------------------
// Core dispatch
// ---------------------------------------------------------------------------

fn emit_node(node: &RenderNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el, bindings, inside_for),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { {#e} },
        RenderNode::If(if_node) => emit_if(if_node, bindings, inside_for),
        RenderNode::For(for_node) => emit_for(for_node, bindings),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes
                .iter()
                .map(|n| emit_node(n, bindings, inside_for))
                .collect();
            quote! { #(#tokens)* }
        }
    }
}

// ---------------------------------------------------------------------------
// Event handler wrapper — shadow-clones captured idents before move
// ---------------------------------------------------------------------------

fn wrap_event_handler(handler_expr: &syn::Expr) -> TokenStream {
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

// ---------------------------------------------------------------------------
// If / else-if / else
// ---------------------------------------------------------------------------

fn emit_if(if_node: &IfNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = if inside_for {
        emit_if_inline(if_node, bindings, true)
    } else {
        let id = next_extract_id();
        let name = quote::format_ident!("__quoin_if_{}", id);
        let closure = emit_if_closure_body(if_node, bindings, false);
        bindings.push(quote! { let #name = ::std::rc::Rc::new(#closure); });
        quote! { { (*#name)() } }
    };
    wrap_with_cfg(&if_node.attrs, inner)
}

fn emit_if_inline(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens: Vec<TokenStream> = if_node
        .then_branch
        .iter()
        .map(|n| emit_node(n, bindings, inside_for))
        .collect();
    let then_view = quote! { #(#then_tokens)* };

    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens: Vec<TokenStream> = else_branch
            .iter()
            .map(|n| emit_node(n, bindings, inside_for))
            .collect();
        let else_view = quote! { #(#else_tokens)* };
        quote! {
            {if #cond {
                ::leptos::prelude::view! { #then_view }.into_any()
            } else {
                ::leptos::prelude::view! { #else_view }.into_any()
            }}
        }
    } else {
        quote! {
            {#cond.then(|| ::leptos::prelude::view! { #then_view }.into_any())}
        }
    }
}

fn emit_if_closure_body(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens: Vec<TokenStream> = if_node
        .then_branch
        .iter()
        .map(|n| emit_node(n, bindings, inside_for))
        .collect();
    let then_view = quote! { #(#then_tokens)* };

    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens: Vec<TokenStream> = else_branch
            .iter()
            .map(|n| emit_node(n, bindings, inside_for))
            .collect();
        let else_view = quote! { #(#else_tokens)* };
        quote! {
            || if #cond {
                ::leptos::prelude::view! { #then_view }.into_any()
            } else {
                ::leptos::prelude::view! { #else_view }.into_any()
            }
        }
    } else {
        quote! {
            || #cond.then(|| ::leptos::prelude::view! { #then_view }.into_any())
        }
    }
}

// ---------------------------------------------------------------------------
// For
// ---------------------------------------------------------------------------

fn emit_for(for_node: &ForNode, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let inner = emit_for_inner(for_node, bindings);
    wrap_with_cfg(&for_node.attrs, inner)
}

fn emit_for_inner(for_node: &ForNode, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body_tokens: Vec<TokenStream> = for_node
        .body
        .iter()
        .map(|n| emit_node(n, bindings, true))
        .collect();
    let body_view = quote! { #(#body_tokens)* };

    let iter_id = next_extract_id();
    let iter_name = quote::format_ident!("__quoin_for_{}", iter_id);
    bindings.push(quote! { let #iter_name = #iterable.clone(); });

    quote! {
        {
            #iter_name.iter().map(|#pat| {
                ::leptos::prelude::view! { #body_view }
            }).collect::<Vec<_>>()
        }
    }
}

// ---------------------------------------------------------------------------
// Element dispatch
// ---------------------------------------------------------------------------

fn emit_element(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = emit_element_inner(el, bindings, inside_for);
    wrap_with_cfg(&el.attrs, inner)
}

fn emit_element_inner(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el, bindings, inside_for),
        "dropdown_menu" => {
            let children_tokens: Vec<TokenStream> = el
                .children
                .iter()
                .map(|c| emit_node(c, bindings, inside_for))
                .collect();
            quote! { <div> #(#children_tokens)* </div> }
        }
        "virtual_list" => {
            let children_tokens: Vec<TokenStream> = el
                .children
                .iter()
                .map(|c| emit_node(c, bindings, inside_for))
                .collect();
            quote! { <div style="overflow-y: auto"> #(#children_tokens)* </div> }
        }
        "clipboard_button" => emit_clipboard_button(el, bindings, inside_for),
        _ => emit_html_tag(
            el,
            match name_str.as_str() {
                "div" => "div",
                "h1" => "h1",
                "h2" => "h2",
                "h3" => "h3",
                "p" | "text" => "p",
                "button" => "button",
                "input" => "input",
                _ => "div",
            },
            bindings,
            inside_for,
        ),
    }
}

// ---------------------------------------------------------------------------
// Clipboard button (Leptos: uses web_sys clipboard API via quoin helper)
// ---------------------------------------------------------------------------

fn emit_clipboard_button(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let copy_text = el
        .args
        .iter()
        .find(|a| a.key == "copy_text")
        .map(|a| &a.value);
    match copy_text {
        Some(ct) => {
            let clip_id = next_extract_id();
            let clip_name = quote::format_ident!("__quoin_clip_{}", clip_id);
            bindings.push(quote! {
                let #clip_name = {
                    let __ct: String = (#ct).to_string();
                    move |_: leptos::ev::MouseEvent| {
                        quoin::clipboard_write_text(&__ct);
                    }
                };
            });
            let mut attrs: Vec<TokenStream> = Vec::new();
            for arg in &el.args {
                let key_str = arg.key.to_string();
                let value = &arg.value;
                match key_str.as_str() {
                    "class" => attrs.push(quote! { class=#value }),
                    "id" => attrs.push(quote! { id=#value }),
                    "disabled" => attrs.push(quote! { disabled=#value }),
                    _ => {}
                }
            }
            let mut children: Vec<TokenStream> = Vec::new();
            for child in &el.children {
                children.push(emit_node(child, bindings, inside_for));
            }
            let tag_ident = proc_macro2::Ident::new("button", proc_macro2::Span::call_site());
            if children.is_empty() {
                quote! { <#tag_ident #(#attrs)* on:click=#clip_name /> }
            } else {
                quote! { <#tag_ident #(#attrs)* on:click=#clip_name> #(#children)* </#tag_ident> }
            }
        }
        None => emit_html_tag(el, "button", bindings, inside_for),
    }
}

// ---------------------------------------------------------------------------
// Generic HTML tag
// ---------------------------------------------------------------------------

fn emit_html_tag(
    el: &Element,
    tag: &str,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let mut attrs = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "id" => attrs.push(quote! { id=#value }),
            "placeholder" => attrs.push(quote! { placeholder=#value }),
            "disabled" => attrs.push(quote! { disabled=#value }),
            "on_click" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:click=#handler })
            }
            "on_mouse_down" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:mousedown=#handler })
            }
            "on_input" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:input=#handler })
            }
            "on_change" => {
                let handler = wrap_event_handler(value);
                attrs.push(quote! { on:change=#handler })
            }
            "value" => {
                if tag == "input" {
                    attrs.push(quote! { value={#value.get()} });
                } else {
                    attrs.push(quote! { value={#value} });
                }
            }
            _ => {}
        }
    }

    let mut children = Vec::new();
    if let Some(children_expr) = &el.children_expr {
        children.push(quote! { {#children_expr} });
    } else {
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    if children.is_empty() {
        quote! { <#tag_ident #(#attrs)* /> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children)* </#tag_ident> }
    }
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

    // Extract parameter names from the user's closure (e.g., `i` in `|i| ...`)
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

    // Get idents referenced in the closure body (excluding params).
    // These need to be cloned before the move closure so each tab's
    // closure captures its own independent copy.
    let body_idents: Vec<proc_macro2::Ident> = collect_handler_idents(on_click_expr)
        .into_iter()
        .filter(|id| !param_names.contains(&id.to_string()))
        .collect();

    // Add `move` to the user's closure so it owns its captures.
    // This means each tab's closure is independent — no shared borrows.
    let on_click_with_move = force_move_on_closure(on_click_expr);

    let tab_labels: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "tab" {
                    let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                    let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;

                    // Shadow param names with literal tab index (e.g., let i = 0)
                    let param_shadows: Vec<TokenStream> = param_idents
                        .iter()
                        .map(|id| quote! { let #id = #index; })
                        .collect();
                    // Clone body idents before creating the move closure
                    let clone_shadows: Vec<TokenStream> = body_idents
                        .iter()
                        .map(|id| quote! { let #id = #id.clone(); })
                        .collect();
                    // Args to pass when calling the closure
                    let call_args: Vec<TokenStream> =
                        param_idents.iter().map(|id| quote! { #id }).collect();

                    return Some(quote! {
                        <li
                            class={move || if #index == #active_expr { "active" } else { "" }}
                            on:click={
                                #(#param_shadows)*
                                #(#clone_shadows)*
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#(#call_args)*) }
                            }
                        >#label</li>
                    });
                }
            }
            None
        })
        .collect();

    quote! { <ul class="tabs"> #(#tab_labels)* </ul> }
}

// ---------------------------------------------------------------------------
// Data table
// ---------------------------------------------------------------------------

fn emit_data_table(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let rows_expr = el.args.iter().find(|a| a.key == "rows").map(|a| &a.value);
    let _on_sort = el
        .args
        .iter()
        .find(|a| a.key == "on_sort")
        .map(|a| &a.value);
    let striped = find_arg_bool(el, "striped");

    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells: Vec<TokenStream> = Vec::new();
    let mut row_cells: Vec<TokenStream> = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c {
            if e.name != "column" {
                continue;
            }

            let label = e
                .args
                .iter()
                .find(|a| a.key == "label")
                .map(|a| &a.value)
                .unwrap_or(&empty_label);
            let _key = e.args.iter().find(|a| a.key == "key").map(|a| &a.value);
            let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);

            let mut th_attrs = vec![quote! { class="px-3 py-2 text-gray-400 font-medium" }];
            if let Some(w) = width {
                th_attrs.push(quote! { style=format!("width: {}px", #w) });
            }
            header_cells.push(quote! { <th #(#th_attrs)*>#label</th> });

            let render_closure = e.args.iter().find(|a| a.key == "render").map(|a| &a.value);
            let col_id = next_extract_id();
            let render_name = quote::format_ident!("__quoin_col_{}", col_id);

            if let Some(rc) = render_closure {
                bindings.push(quote! { let #render_name = ::std::rc::Rc::new(#rc); });
                let mut td_attrs = vec![quote! { class="px-3 py-2 text-white" }];
                if let Some(w) = width {
                    td_attrs.push(quote! { style=format!("width: {}px", #w) });
                }
                row_cells.push(quote! { <td #(#td_attrs)*>{(#render_name)(&__row)}</td> });
            } else {
                row_cells.push(quote! { <td class="px-3 py-2 text-white"></td> });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let striped_class = if striped { " table-striped" } else { "" };

    quote! {
        <table class={concat!("w-full", #striped_class)}>
            <thead><tr> #(#header_cells)* </tr></thead>
            <tbody>
                {#rows.iter().map(|__row| {
                    ::leptos::prelude::view! { <tr> #(#row_cells)* </tr> }
                }).collect::<Vec<_>>()}
            </tbody>
        </table>
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn wrap_with_cfg(attrs: &[syn::Attribute], inner: TokenStream) -> TokenStream {
    let cfg_attrs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    if cfg_attrs.is_empty() {
        inner
    } else {
        quote! { { #(#cfg_attrs)* { #inner } } }
    }
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
