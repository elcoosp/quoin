use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::tailwind::{TranspiledStyles, transpile_class};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

// ---------------------------------------------------------------------------
// Ident collection (skips closure bodies to avoid collecting params/locals)
// ---------------------------------------------------------------------------

struct PathIdentCollector(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollector {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        if expr_path.path.segments.len() == 1 && expr_path.path.leading_colon.is_none() {
            if let Some(seg) = expr_path.path.segments.last() {
                self.0.push(seg.ident.clone());
            }
        }
        syn::visit::visit_expr_path(self, expr_path);
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}
}

fn collect_handler_idents(expr: &Expr) -> Vec<proc_macro2::Ident> {
    let mut collector = PathIdentCollector(vec![]);
    collector.visit_expr(expr);
    collector
        .0
        .sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    collector.0.dedup_by(|a, b| a.to_string() == b.to_string());
    collector.0
}

// ---------------------------------------------------------------------------
// StripMove visitor
// ---------------------------------------------------------------------------

struct StripMove;

impl VisitMut for StripMove {
    fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
        closure.movability = None;
    }
}

fn strip_move_from_closure(expr: &Expr) -> syn::Expr {
    let mut expr = expr.clone();
    StripMove.visit_expr_mut(&mut expr);
    expr
}

// ---------------------------------------------------------------------------
// emit_render entry point
// ---------------------------------------------------------------------------

pub fn emit_render(node: &RenderNode) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => quote! { #e },
        RenderNode::If(if_node) => emit_if(if_node),
        RenderNode::For(for_node) => emit_for(for_node),
        RenderNode::Root(nodes) => {
            let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
            quote! { gpui::div().children(vec![#(#node_tokens),*]) }
        }
    }
}

// ---------------------------------------------------------------------------
// Element emitters
// ---------------------------------------------------------------------------

fn emit_element(el: &Element) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "button" => emit_button(el),
        "input" => emit_input(el),
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el),
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

    for child in &el.children {
        let child_elem = emit_render(child);
        chain = quote! { #chain.child(#child_elem) };
    }

    if let Some(handler_expr) = find_arg_expr(el, "on_click") {
        let idents = collect_handler_idents(handler_expr);
        let shadows: Vec<TokenStream> = idents
            .iter()
            .map(|id| {
                quote! { let #id = #id.clone(); }
            })
            .collect();
        let handler_no_move = strip_move_from_closure(handler_expr);
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, {
                #(#shadows)*
                let __handler = ::std::rc::Rc::new(#handler_no_move);
                move |_, _, _| {
                    __handler(())
                }
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

    let has_class = find_arg_expr(el, "class").is_some();
    let mut wrapper_styles = quote! {};

    if let Some(class_expr) = find_arg_expr(el, "class") {
        if let Some(styles) = try_transpile_class(class_expr) {
            for style in styles.normal {
                wrapper_styles = quote! { #wrapper_styles #style };
            }
        }
    }

    let input_construction = if has_class {
        quote! {
            gpui::div()
                #wrapper_styles
                .child(quoin_ui_gpui::Input::new(&__entity).appearance(false))
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
                let __initial_val: String = self.#value_ident.get();
                let __entity = cx.new::<quoin_ui_gpui::InputState>(|cx| {
                    let mut __state = quoin_ui_gpui::InputState::new(window, cx);
                    __state.set_placeholder(#placeholder, window, cx);
                    __state.set_value(__initial_val, window, cx);
                    __state
                });
                let __sub = cx.observe(&__entity, |this, __entity, cx| {
                    let __new_val: String = __entity.read(cx).value().to_string();
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

fn emit_tabs(el: &Element) -> TokenStream {
    let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
    let on_click_expr = find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

    let on_click_no_move = strip_move_from_closure(on_click_expr);

    let tab_labels: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "tab" {
                    let label = find_arg_string(e, "label").unwrap_or_default();
                    let index = find_arg_expr(e, "index").expect("tab requires 'index'");
                    return Some(quote! {
                        ( #index, #label.to_string() )
                    });
                }
            }
            None
        })
        .collect();

    quote! {
        {
            let __active = #active_expr;
            let __on_click = ::std::rc::Rc::new(#on_click_no_move);
            let __labels: Vec<(usize, String)> = vec![#(#tab_labels),*];
            let __tab_elements: Vec<gpui::AnyElement> = __labels.iter().map(|(idx, label)| {
                let __is_active = *idx == __active;
                let mut __el = gpui::div()
                    .px(gpui::px(16.0))
                    .py(gpui::px(8.0))
                    .cursor_pointer()
                    .child(label.clone());

                if __is_active {
                    __el = __el.text_color(gpui::white());
                } else {
                    __el = __el.text_color(gpui::rgb(0x9ca3af));
                }

                let __idx = *idx;
                let __tab_on_click = __on_click.clone();
                __el.on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                    __tab_on_click(__idx)
                }).into_any_element()
            }).collect();

            gpui::div().flex().children(__tab_elements)
        }
    }
}

fn emit_data_table(el: &Element) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows").expect("data_table requires 'rows'");

    let header_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c {
                if e.name == "column" {
                    let label = find_arg_string(e, "label").unwrap_or_default();
                    return Some(quote! {
                        gpui::div()
                            .px(gpui::px(12.0))
                            .py(gpui::px(8.0))
                            .text_color(gpui::rgb(0x6b7280))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(#label.to_string())
                            .into_any_element()
                    });
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
                        find_arg_expr(e, "render").expect("column requires 'render' closure");
                    return Some(quote! {
                        gpui::div()
                            .w_full()
                            .px(gpui::px(12.0))
                            .py(gpui::px(8.0))
                            .text_color(gpui::rgb(0xffffff))
                            .child((#render_closure)(&__row))
                            .into_any_element()
                    });
                }
            }
            None
        })
        .collect();

    quote! {
        {
            let __rows = #rows_expr;
            let __header = gpui::div().flex().children(vec![#(#header_cells),*]);

            let __row_elements: Vec<gpui::AnyElement> = __rows.iter().map(|__row| {
                gpui::div().flex().children(vec![#(#row_cells),*]).into_any_element()
            }).collect();

            gpui::div()
                .flex_col()
                .gap_1()
                .size_full()
                .child(__header)
                .children(__row_elements)
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

    if let Some(handler_expr) = find_arg_expr(el, "on_click") {
        let idents = collect_handler_idents(handler_expr);
        let shadows: Vec<TokenStream> = idents
            .iter()
            .map(|id| {
                quote! { let #id = #id.clone(); }
            })
            .collect();
        let handler_no_move = strip_move_from_closure(handler_expr);
        chain = quote! {
            #chain.on_mouse_down(gpui::MouseButton::Left, {
                #(#shadows)*
                let __handler = ::std::rc::Rc::new(#handler_no_move);
                move |_, _, _| {
                    __handler(())
                }
            })
        };
    }

    chain
}

fn emit_if(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_branch = emit_nodes(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        let else_tokens = if else_branch.len() == 1 {
            match &else_branch[0] {
                RenderNode::If(nested_if) => emit_if(nested_if),
                _ => emit_nodes(else_branch),
            }
        } else {
            emit_nodes(else_branch)
        };
        quote! { { if #cond { #then_branch } else { #else_tokens } } }
    } else {
        quote! { { if #cond { #then_branch } } }
    }
}

fn emit_for(for_node: &ForNode) -> TokenStream {
    let pat = &for_node.pat;
    let iterable = &for_node.iterable;
    let body = emit_nodes(&for_node.body);
    quote! {
        {
            gpui::div().children(
                #iterable.into_iter().map(|#pat| {
                    #body
                }).collect::<Vec<_>>()
            )
        }
    }
}

fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    let node_tokens: Vec<TokenStream> = nodes.iter().map(emit_render).collect();
    quote! { gpui::div().children(vec![#(#node_tokens),*]) }
}

fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|a| a.key == key).map(|a| &a.value)
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
