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
    let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();

    let value_expr = match find_arg_expr(el, "value") {
        Some(e) => e,
        None => {
            let mut chain = quote! {
                gpui::div()
                    .rounded(gpui::px(6.0))
                    .px(gpui::px(12.0))
                    .py(gpui::px(8.0))
                    .bg(gpui::rgb(0x1f2937))
                    .border_1()
                    .border_color(gpui::rgb(0x374151))
                    .text_color(gpui::rgb(0x9ca3af))
                    .child(#placeholder)
            };
            if let Some(class_expr) = find_arg_expr(el, "class") {
                if let Some(styles) = try_transpile_class(class_expr) {
                    for style in styles.normal {
                        chain = quote! { #chain #style };
                    }
                }
            }
            return chain;
        }
    };

    // Extract the last segment ident to handle both `filter_text` and `self.filter_text`
    let value_ident = match value_expr {
        Expr::Path(path) => path.path.segments.last().map(|seg| seg.ident.clone()),
        Expr::Field(field) => match &field.member {
            syn::Member::Named(ident) => Some(ident.clone()),
            _ => None,
        },
        _ => None,
    };
    let value_ident = match value_ident {
        Some(id) => id,
        None => {
            let mut chain = quote! {
                gpui::div()
                    .rounded(gpui::px(6.0))
                    .px(gpui::px(12.0))
                    .py(gpui::px(8.0))
                    .bg(gpui::rgb(0x1f2937))
                    .border_1()
                    .border_color(gpui::rgb(0x374151))
                    .text_color(gpui::rgb(0xffffff))
                    .child(#value_expr)
            };
            if let Some(class_expr) = find_arg_expr(el, "class") {
                if let Some(styles) = try_transpile_class(class_expr) {
                    for style in styles.normal {
                        chain = quote! { #chain #style };
                    }
                }
            }
            return chain;
        }
    };

    let input_id_str = format!("__quoin_input_{}", value_ident);

    let wrap_in_div = find_arg_expr(el, "class").is_some();
    let mut wrapper_styles = quote! {};
    if let Some(class_expr) = find_arg_expr(el, "class") {
        if let Some(styles) = try_transpile_class(class_expr) {
            for style in styles.normal {
                wrapper_styles = quote! { #wrapper_styles #style };
            }
        }
    }

    let _ = el.args.iter().find(|(k, _)| k == "on_input");

    let input_construction = if wrap_in_div {
        quote! {
            gpui::div()
                #wrapper_styles
                .child(quoin_ui_gpui::Input::new(&__entity))
        }
    } else {
        quote! {
            quoin_ui_gpui::Input::new(&__entity)
        }
    };

    quote! {
        {
            let __input_id: &str = #input_id_str;
            if !self._quoin_inputs.contains(__input_id) {
                // Read initial value in render scope where `self` is available
                let __initial_val: String = self.#value_ident.get();
                let __entity = cx.new::<quoin_ui_gpui::InputState>(|cx| {
                    let mut __state = quoin_ui_gpui::InputState::new(window, cx);
                    __state.set_placeholder(#placeholder, window, cx);
                    __state.set_value(__initial_val, window, cx);
                    __state
                });
                let __sub = cx.observe(&__entity, |this, __entity, cx| {
                    let __new_val: String = __entity.read(cx).value().to_string();
                    // In the observe closure, `this` is `&mut Self`
                    this.#value_ident.set(__new_val);
                });
                self._quoin_inputs.insert(__input_id.to_string(), __entity, __sub);
            } else {
                let __entity = self._quoin_inputs.get(__input_id).unwrap();
                let __current: String = __entity.read(cx).value().to_string();
                let __desired: String = self.#value_ident.get();
                if __current != __desired {
                    __entity.update(cx, |__state, cx| {
                        __state.set_value(__desired, window, cx);
                    });
                }
            }
            let __entity = self._quoin_inputs.get(__input_id).unwrap().clone();
            #input_construction
        }
    }
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
