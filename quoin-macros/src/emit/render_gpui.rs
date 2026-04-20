use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::tailwind::{TranspiledStyles, transpile_class};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { #e },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
    }
}

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "button" => emit_button(el),
        "input" => emit_input(el),
        _ => emit_generic_element(el),
    }
}

fn emit_button(el: &Element) -> TokenStream {
    let mut chain = quote! {
        gpui::div()
            .cursor_pointer()
            .rounded(gpui::px(6.0))
            .px(gpui::px(8.0))
            .py(gpui::px(8.0))
            .flex()
            .items_center()
            .justify_center()
            .text_color(gpui::white())
    };

    let primary = find_arg_bool(el, "primary");
    let ghost = find_arg_bool(el, "ghost");
    let destructive = find_arg_bool(el, "destructive");

    if primary {
        chain = quote! { #chain.bg(gpui::rgb(0x2563eb)) };
    } else if destructive {
        chain = quote! { #chain.bg(gpui::rgb(0xdc2626)) };
    } else if !ghost {
        chain = quote! { #chain.bg(gpui::rgb(0x4e4e4e)) };
    }

    for child in &el.children {
        let child_elem = emit_render(child);
        chain = quote! { #chain.child(#child_elem) };
    }

    if let Some(class_expr) = find_arg_expr(el, "class") {
        if let Some(styles) = try_transpile_class(class_expr) {
            for style in styles.normal {
                chain = quote! { #chain #style };
            }
            if !styles.hover.is_empty() {
                let hover_tokens = styles.hover;
                chain = quote! { #chain.hover(|__s| __s #(#hover_tokens)*) };
            }
        }
    }

    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_click") {
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, {
                let __handler = #handler_expr;
                move |_, _, _| __handler(())
            })
        };
    }

    chain
}

fn emit_input(el: &Element) -> TokenStream {
    let placeholder = find_arg_string(el, "placeholder");
    let value_str = find_arg_string(el, "value").unwrap_or_default();

    let display = if value_str.is_empty() {
        placeholder.unwrap_or_default()
    } else {
        value_str
    };

    let mut chain = quote! {
        gpui::div()
            .rounded(gpui::px(6.0))
            .px(gpui::px(12.0))
            .py(gpui::px(8.0))
            .bg(gpui::rgb(0xffffff))
            .text_color(gpui::rgb(0x111827))
            .child(#display)
    };

    if let Some(class_expr) = find_arg_expr(el, "class") {
        if let Some(styles) = try_transpile_class(class_expr) {
            for style in styles.normal {
                chain = quote! { #chain #style };
            }
        }
    }

    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_input") {
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, {
                let __handler = #handler_expr;
                move |_, _, _| __handler(())
            })
        };
    }

    chain
}

fn emit_generic_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    let mut chain = match name_str.as_str() {
        "div" => quote! { gpui::div() },
        "h1" => quote! { gpui::div().text_xl().font_weight(gpui::FontWeight::BOLD) },
        "h2" => quote! { gpui::div().text_lg().font_weight(gpui::FontWeight::BOLD) },
        "p" | "text" => quote! { gpui::div() },
        _ => quote! { gpui::div() },
    };

    if let Some(class_expr) = find_arg_expr(el, "class") {
        if let Some(styles) = try_transpile_class(class_expr) {
            for style in styles.normal {
                chain = quote! { #chain #style };
            }
            if !styles.hover.is_empty() {
                let hover_tokens = styles.hover;
                chain = quote! { #chain.hover(|__s| __s #(#hover_tokens)*) };
            }
        }
    }

    if let Some(children_expr) = &el.children_expr {
        chain = quote! { #chain.children(#children_expr) };
    } else {
        for child in &el.children {
            let child_elem = emit_render(child);
            chain = quote! { #chain.child(#child_elem) };
        }
    }

    if let Some((_, handler_expr)) = el.args.iter().find(|(k, _)| k == "on_click") {
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, {
                let __handler = #handler_expr;
                move |_, _, _| __handler(())
            })
        };
    }

    chain
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_branch = emit_nodes(else_branch);
        quote! { if #cond { #then_branch } else { #else_branch } }
    } else {
        quote! { if #cond { #then_branch } }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes(&for_node.body);
    quote! {
        .children(
            #iterable.into_iter().map(|#pat| {
                #body
            })
        )
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<_> = nodes.iter().map(emit_render).collect();
    quote! { gpui::div().children(vec![#(#node_tokens),*]) }
}

fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn find_arg_string(el: &Element, key: &str) -> Option<String> {
    find_arg_expr(el, key).and_then(|e| {
        if let Expr::Lit(expr_lit) = e {
            if let syn::Lit::Str(s) = &expr_lit.lit {
                Some(s.value())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn find_arg_bool(el: &Element, key: &str) -> bool {
    find_arg_expr(el, key)
        .map(|e| {
            if let Expr::Lit(expr_lit) = e {
                if let syn::Lit::Bool(b) = &expr_lit.lit {
                    return b.value;
                }
            }
            false
        })
        .unwrap_or(false)
}

fn try_transpile_class(expr: &Expr) -> Option<TranspiledStyles> {
    if let Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
            return Some(transpile_class(&lit_str.value()));
        }
    }
    None
}
