//! Stateful widgets and simple elements for the Xilem backend.
//!
//! Every element shares the argument contract with the other backends.
//! Anything Xilem cannot support natively emits a precise `compile_error!`
//! instead of silently degrading.

use crate::emit::common::{find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, generic, handler, style};

/// `compile_error!` for an element the Xilem backend does not support yet.
pub(crate) fn unsupported_element(name: &str, why: &str) -> TokenStream {
    let message = format!("`{name}` is not supported by the Xilem backend: {why}");
    quote! { ::core::compile_error!(#message) }
}

// ── separator ───────────────────────────────────────────────────────────────

/// `separator` → a 1px rule styled with the `border` theme token.
pub(crate) fn emit_separator(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let _ = ctx;
    let class = find_arg_string(el, "class");
    let rule = quote! {
        ::xilem::view::sized_box(::xilem::view::label(""))
            .height(::xilem::masonry::layout::Dim::Fixed(
                ::xilem::masonry::layout::Length::px(1.0),
            ))
            .background(::quoin::xilem_theme::border())
    };
    match class.as_deref() {
        Some(c) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(c, &mut ctx.text);
            ctx.text = saved;
            let mut out = rule;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => rule,
    }
}

// ── checkbox / switch ───────────────────────────────────────────────────────

/// `checkbox(checked: <bool signal>, on_change: handler)`.
pub(crate) fn emit_checkbox(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    emit_toggle(el, ctx, false)
}

/// `switch` → Xilem's `switch` view.
pub(crate) fn emit_switch(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    emit_toggle(el, ctx, true)
}

fn emit_toggle(el: &Element, ctx: &mut super::EmitterCtx, switch: bool) -> TokenStream {
    let checked_fallback: syn::Expr =
        syn::parse_quote!((|| unreachable!("checkbox/switch requires `checked:`"))());
    let checked_expr = find_arg_expr(el, "checked").unwrap_or(&checked_fallback);

    // The checked value is re-read per rebuild; the callback owns a clone.
    let value_shadows = handler::capture_shadows(checked_expr, ctx);

    let (handler_setup, call) =
        match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_checked_change")) {
            Some(handler_expr) => {
                let (block, name) = handler::handler_block(handler_expr, ctx);
                let call = if handler::handler_arity_is_zero(handler_expr) {
                    quote! { #name(); }
                } else {
                    quote! { #name(__new); }
                };
                (block, call)
            }
            None => (quote! {}, quote! {}),
        };

    let label = find_arg_string(el, "label").unwrap_or_default();

    let view = if switch {
        quote! {
            {
                #(#value_shadows)*
                let __value = ::core::clone::Clone::clone(&(#checked_expr));
                #handler_setup
                let __cb_value = ::core::clone::Clone::clone(&__value);
                ::xilem::view::switch(
                    ::quoin::Signal::get(&__value),
                    move |__s, __new: bool| {
                        let _ = __s;
                        ::quoin::Signal::set(&__cb_value, __new);
                        #call
                    },
                )
            }
        }
    } else {
        quote! {
            {
                #(#value_shadows)*
                let __value = ::core::clone::Clone::clone(&(#checked_expr));
                #handler_setup
                let __cb_value = ::core::clone::Clone::clone(&__value);
                ::xilem::view::checkbox(
                    #label,
                    ::quoin::Signal::get(&__value),
                    move |__s, __new: bool| {
                        let _ = __s;
                        ::quoin::Signal::set(&__cb_value, __new);
                        #call
                    },
                )
            }
        }
    };

    let disabled = if matches!(
        find_arg_expr(el, "disabled"),
        Some(syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. })) if b.value
    ) {
        quote! { .disabled(true) }
    } else {
        quote! {}
    };

    let styled = match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = view;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => view,
    };

    quote! { #styled #disabled }
}

// ── slider ──────────────────────────────────────────────────────────────────

/// `slider(value, min, max, on_change)` → Xilem's `slider`.
pub(crate) fn emit_slider(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let value_fallback: syn::Expr = syn::parse_quote!(0.0);
    let min_fallback: syn::Expr = syn::parse_quote!(0.0);
    let max_fallback: syn::Expr = syn::parse_quote!(100.0);
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
    let min_expr = find_arg_expr(el, "min").unwrap_or(&min_fallback);
    let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);

    let (handler_setup, call) =
        match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input")) {
            Some(handler_expr) => {
                let (block, name) = handler::handler_block(handler_expr, ctx);
                let call = if handler::handler_arity_is_zero(handler_expr) {
                    quote! { #name(); }
                } else {
                    quote! { #name(__new); }
                };
                (block, call)
            }
            None => (quote! {}, quote! {}),
        };

    let base = quote! {
        {
            #handler_setup
            ::xilem::view::slider(
                #min_expr,
                #max_expr,
                ::core::clone::Clone::clone(&(#value_expr)),
                move |__s, __new: f64| {
                    let _ = __s;
                    #call
                },
            )
        }
    };

    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            style::wrap_errors(base, &lowered)
        }
        None => base,
    }
}

// ── scroll_area ─────────────────────────────────────────────────────────────

/// `scroll_area(direction: "..")` → Xilem's `portal`.
pub(crate) fn emit_scroll_area(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let _ = find_arg_string(el, "direction");
    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, inside_for));
    }
    let child = match kids.len() {
        0 => quote! { ::xilem::view::sized_box(::xilem::view::label("")) },
        1 => kids[0].clone(),
        _ => generic::flex_sequence(&kids, quoin_style::FlexDir::Column),
    };
    let scrolled = quote! { ::xilem::view::portal(#child) };
    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = scrolled;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => scrolled,
    }
}

// ── virtual_list ────────────────────────────────────────────────────────────

/// `virtual_list(id, count, item_height, render)` → Xilem `virtual_scroll`.
pub(crate) fn emit_virtual_list(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let count_fallback: syn::Expr = syn::parse_quote!(0);
    let count_expr = find_arg_expr(el, "count").unwrap_or(&count_fallback);
    let _item_height = find_arg_f32(el, "item_height").unwrap_or(32.0);
    let render_fallback: syn::Expr = syn::parse_quote!((|__ix: usize| ::xilem::view::label("")));
    let render_expr = find_arg_expr(el, "render").unwrap_or(&render_fallback);

    // The render closure is `Fn(&mut State, usize) -> ChildView`; captures
    // shadow outside it.
    let mut render_pats = std::collections::HashSet::new();
    render_pats.insert("__ix".to_owned());
    let _ = &mut render_pats;
    let idents = crate::transpile::collect_handler_idents_excluding_params(render_expr);
    let render_shadows: Vec<TokenStream> = idents
        .iter()
        .filter(|id| *id != "__ix")
        .map(|id| {
            ctx.capture(id);
            quote! { let #id = ::core::clone::Clone::clone(&#id); }
        })
        .collect();

    quote! {
        {
            #(#render_shadows)*
            ::xilem::view::virtual_scroll(
                #count_expr,
                #render_expr,
            )
        }
    }
}

// ── progress ────────────────────────────────────────────────────────────────

/// `progress(value, max)` → a disabled slider used as a bar.
pub(crate) fn emit_progress(el: &Element, _ctx: &mut super::EmitterCtx) -> TokenStream {
    let value_fallback: syn::Expr = syn::parse_quote!(0.0);
    let max_fallback: syn::Expr = syn::parse_quote!(100.0);
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
    let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);
    quote! {
        ::xilem::view::slider(
            0.0,
            #max_expr,
            ::core::clone::Clone::clone(&(#value_expr)),
            move |__s, __v: f64| { let _ = (__s, __v); },
        )
        .disabled(true)
    }
}

// ── tabs ────────────────────────────────────────────────────────────────────

/// `tabs(active: <usize signal>, on_click) { tab(label: "..") { … } }` —
/// a flex strip of text buttons plus a boxed, branch-selected content pane.
pub(crate) fn emit_tabs(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let active_fallback: syn::Expr = syn::parse_quote!(0usize);
    let active_expr = find_arg_expr(el, "active").unwrap_or(&active_fallback);

    let mut labels = Vec::new();
    let mut bodies = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(tab_el) = child
            && tab_el.name == "tab"
        {
            labels.push(find_arg_string(tab_el, "label").unwrap_or_default());
            let mut tab_kids = Vec::new();
            for tab_child in &tab_el.children {
                tab_kids.push(emit_node(tab_child, ctx, inside_for));
            }
            bodies.push(match tab_kids.len() {
                0 => quote! { ::xilem::view::sized_box(::xilem::view::label("")) },
                1 => tab_kids[0].clone(),
                _ => generic::flex_sequence(&tab_kids, quoin_style::FlexDir::Column),
            });
        }
    }

    let label_lits: Vec<TokenStream> = labels.iter().map(|l| quote! { #l, }).collect();
    let tab_count = labels.len();
    let label_array = quote! { const LABELS: &[&str] = &[#(#label_lits)*]; };
    let strip_buttons: Vec<TokenStream> = (0..tab_count)
        .map(|index| {
            let lit = proc_macro2::Literal::usize_unsuffixed(index);
            quote! {
                ::xilem::view::text_button(
                    LABELS[#lit],
                    move |__s| {
                        let _ = __s;
                        ::quoin::Signal::set(&(#active_expr), #lit);
                    },
                )
            }
        })
        .collect();

    let content_arms: Vec<TokenStream> = bodies
        .iter()
        .enumerate()
        .map(|(index, body)| {
            let lit = proc_macro2::Literal::usize_unsuffixed(index);
            quote! { #lit => #body.boxed() }
        })
        .collect();

    let strip = generic::flex_sequence(&strip_buttons, quoin_style::FlexDir::Row);
    let content = quote! {
        {
            #label_array
            let __active: usize = ::quoin::Signal::get(&(#active_expr));
            match __active.min(#tab_count.saturating_sub(1)) {
                #(#content_arms)*
                _ => ::xilem::view::sized_box(::xilem::view::label("")).boxed(),
            }
        }
    };

    let whole = generic::flex_sequence(&[strip, content], quoin_style::FlexDir::Column);
    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = whole;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => whole,
    }
}

// ── data_table ──────────────────────────────────────────────────────────────

/// `data_table(rows: <expr>, striped) { column(label: "..", render: |row| …) }`
/// → a header flex row plus a `for`-style body of row flexes.
pub(crate) fn emit_data_table(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let rows_fallback: syn::Expr = syn::parse_quote!(::std::vec::Vec::<()>::new());
    let rows_expr = find_arg_expr(el, "rows").unwrap_or(&rows_fallback);
    let _striped = matches!(
        find_arg_expr(el, "striped"),
        Some(syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. })) if b.value
    );

    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(col) = child
            && col.name == "column"
        {
            let label = find_arg_string(col, "label").unwrap_or_default();
            header_cells.push(quote! {
                ::xilem::view::label(#label)
                    .color(::quoin::xilem_theme::muted_foreground_text())
            });
            match find_arg_expr(col, "render") {
                Some(render_expr) => {
                    row_cells.push(quote! { (#render_expr)(&__row) });
                }
                None => row_cells.push(quote! { ::xilem::view::label("") }),
            }
        }
    }

    let header = generic::flex_sequence(&header_cells, quoin_style::FlexDir::Row);
    let cells = &row_cells;

    // Body rows: rebuild through a Vec sequence keyed by index.
    let body = quote! {
        (#rows_expr)
            .into_iter()
            .map(move |__row| {
                let __row = ::core::clone::Clone::clone(&__row);
                ::xilem::view::flex(
                    ::xilem::kurbo::Axis::Horizontal,
                    (#(#cells,)*),
                )
            })
            .collect::<::std::vec::Vec<_>>()
    };

    let whole = generic::flex_sequence(&[header, body], quoin_style::FlexDir::Column);
    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = whole;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => whole,
    }
}

// ── dropdown_menu ───────────────────────────────────────────────────────────

/// `dropdown_menu { item(label: "..", on_click: ..) }` — a column of buttons
/// (the portal-overlay dropdown lands with the M5 push).
pub(crate) fn emit_dropdown_menu(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let mut items = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(item) = child
            && (item.name == "item" || item.name == "button")
        {
            let label = find_arg_string(item, "label").unwrap_or_default();
            let action = if let Some(handler_expr) = find_arg_expr(item, "on_click") {
                handler::handler_to_button_cb(handler_expr, ctx)
            } else {
                quote! { move |__s| { let _ = __s; } }
            };
            items.push(quote! {
                ::xilem::view::text_button(#label, #action)
            });
        }
    }
    if items.is_empty() {
        return unsupported_element("dropdown_menu", "no `item` children found");
    }
    let stack = generic::flex_sequence(&items, quoin_style::FlexDir::Column);
    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = stack;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => stack,
    }
}

// ── badge / skeleton ────────────────────────────────────────────────────────

/// `badge(color: <expr>)` → a rounded, tinted label.
pub(crate) fn emit_badge(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, inside_for));
    }
    let content = match kids.len() {
        0 => quote! { ::xilem::view::label("") },
        1 => kids[0].clone(),
        _ => generic::flex_sequence(&kids, quoin_style::FlexDir::Row),
    };
    let badge = quote! {
        ::xilem::view::sized_box(#content)
            .padding(::xilem::masonry::properties::Padding::all(
                ::xilem::masonry::layout::Length::px(4.0),
            ))
            .corner_radius(::xilem::masonry::layout::Length::px(9999.0))
            .background(::quoin::xilem_theme::secondary())
    };
    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = badge;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => badge,
    }
}

/// `skeleton` variants → muted placeholder blocks.
pub(crate) fn emit_skeleton(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let _ = el;
    let _ = ctx;
    quote! {
        ::xilem::view::sized_box(::xilem::view::label(""))
            .width(::xilem::masonry::layout::Dim::Stretch)
            .height(::xilem::masonry::layout::Dim::Fixed(
                ::xilem::masonry::layout::Length::px(16.0),
            ))
            .corner_radius(::xilem::masonry::layout::Length::px(4.0))
            .background(::quoin::xilem_theme::muted())
    }
}
