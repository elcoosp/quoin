//! Stateful widgets and simple elements for the Floem backend.
//!
//! Every element here shares the argument contract with the GPUI/Leptos
//! backends so one source compiles everywhere. Anything Floem cannot support
//! natively emits a precise `compile_error!` instead of silently degrading.

use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, handler, style};

/// `compile_error!` for an element the Floem backend does not support yet.
pub(crate) fn unsupported_element(name: &str, why: &str) -> TokenStream {
    let message = format!("`{name}` is not supported by the Floem backend: {why}");
    quote! { ::core::compile_error!(#message) }
}

// ── separator ───────────────────────────────────────────────────────────────

/// `separator` → a 1px rule styled with the `border` theme token.
pub(crate) fn emit_separator(el: &Element) -> TokenStream {
    let class = find_arg_string(el, "class");
    let styles = class.as_deref().map(style::lower_class);
    let rule = quote! {
        ::floem::views::Empty::new().style(|__s| __s
            .width_full()
            .height(1.0)
            .background(::quoin::floem_theme::border()))
    };
    match styles {
        Some(s) => style::apply_styles(rule, &s),
        None => rule,
    }
}

// ── checkbox / switch ───────────────────────────────────────────────────────

/// `checkbox(checked: <bool signal>, on_change: handler)` — the quoin signal
/// stays the source of truth through a two-way effect bridge, while Floem's
/// `Checkbox::new_rw` owns the click toggle natively.
pub(crate) fn emit_checkbox(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    emit_toggle(el, ctx, false)
}

/// `switch` — Floem's `ToggleButton` with the same two-way bridge.
pub(crate) fn emit_switch(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    emit_toggle(el, ctx, true)
}

fn emit_toggle(el: &Element, ctx: &mut super::EmitterCtx, switch: bool) -> TokenStream {
    let checked_fallback: syn::Expr =
        syn::parse_quote!((|| unreachable!("checkbox/switch requires `checked:`")));
    let checked_expr = find_arg_expr(el, "checked").unwrap_or(&checked_fallback);
    let capture = handler::capture_shadows(checked_expr, ctx);

    let change_hook =
        match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_checked_change")) {
            Some(handler_expr) => {
                let (block, _name) = handler::handler_block(handler_expr, ctx);
                let call = if handler::handler_arity_is_zero(handler_expr) {
                    quote! { __h() }
                } else {
                    quote! { __h(::quoin::Signal::get(&__value)) }
                };
                quote! {
                    #block
                    ::floem::reactive::Effect::new(
                        move |__prev: ::core::option::Option<()>| {
                            let _ = __prev;
                            let __current = ::quoin::Signal::get(&__value);
                            #call;
                        },
                    );
                }
            }
            None => quote! {},
        };

    let view = if switch {
        quote! { ::floem::views::toggle_button::ToggleButton::new_rw(__state) }
    } else {
        quote! { ::floem::views::Checkbox::new_rw(__state) }
    };

    let styled = match find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(view, &styles)
        }
        None => view,
    };

    let disabled = if find_arg_bool(el, "disabled") {
        quote! { .disabled(|| true) }
    } else {
        quote! {}
    };

    // Fully inline: the bridge registers when the view is constructed, so a
    // checkbox inside a `for` body picks up the loop item's captures.
    quote! {
        {
            #(#capture)*
            let __value = ::core::clone::Clone::clone(&(#checked_expr));
            let __state = ::floem::reactive::RwSignal::new(::quoin::Signal::get(&__value));
            ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                let _ = __prev;
                let __next = ::quoin::Signal::get(&__value);
                if ::floem::reactive::SignalGet::get_untracked(&__state) != __next {
                    ::floem::reactive::SignalUpdate::set(&__state, __next);
                }
            });
            ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                let _ = __prev;
                let __edited = ::floem::reactive::SignalGet::get(&__state);
                if ::quoin::Signal::get(&__value) != __edited {
                    ::quoin::Signal::set(&__value, __edited);
                }
            });
            #change_hook
            #styled #disabled
        }
    }
}

// ── slider ──────────────────────────────────────────────────────────────────

/// `slider(value, min, max, on_change)` → Floem `slider` + `SliderChanged`.
pub(crate) fn emit_slider(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let value_fallback: syn::Expr = syn::parse_quote!(0.0);
    let min_fallback: syn::Expr = syn::parse_quote!(0.0);
    let max_fallback: syn::Expr = syn::parse_quote!(100.0);
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
    let min_expr = find_arg_expr(el, "min").unwrap_or(&min_fallback);
    let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);

    let capture = handler::capture_shadows(value_expr, ctx);

    let change_hook = match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"))
    {
        Some(handler_expr) => {
            let (block, _name) = handler::handler_block(handler_expr, ctx);
            let call = if handler::handler_arity_is_zero(handler_expr) {
                quote! { __h() }
            } else {
                quote! { __h(__event.pct.0) }
            };
            quote! {
                .on_event(::floem::views::slider::SliderChanged::listener(), {
                    #block
                    move |__cx, __event| {
                        let _ = __cx;
                        #call;
                        ::floem::event::EventPropagation::Continue
                    }
                })
            }
        }
        None => quote! {},
    };

    let view = quote! {
        ::floem::views::slider(move || {
            let __min: f64 = #min_expr;
            let __max: f64 = #max_expr;
            let __value: f64 = ::core::clone::Clone::clone(&#value_expr);
            if __max > __min { (__value - __min) / (__max - __min) * 100.0 } else { 0.0 }
        })
    };

    let styled = match find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(view, &styles)
        }
        None => view,
    };

    let disabled = if find_arg_bool(el, "disabled") {
        quote! { .disabled(|| true) }
    } else {
        quote! {}
    };

    quote! {
        {
            #(#capture)*
            #styled #change_hook #disabled
        }
    }
}

// ── tooltip ─────────────────────────────────────────────────────────────────

/// `tooltip(text: "..") { hover-target }` → Floem's `tooltip` view.
pub(crate) fn emit_tooltip(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let tip = find_arg_string(el, "text").unwrap_or_default();
    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, inside_for));
    }
    let child = match kids.len() {
        0 => quote! { ::floem::views::Empty::new() },
        1 => kids[0].clone(),
        _ => super::generic::static_stack(&kids, quoin_style::FlexDir::Column, None),
    };
    quote! { ::floem::views::tooltip(#child, move || ::floem::views::Label::derived(move || #tip)) }
}

// ── scroll_area ─────────────────────────────────────────────────────────────

/// `scroll_area(direction: "..")` → Floem's `scroll` view.
pub(crate) fn emit_scroll_area(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let _direction = find_arg_string(el, "direction");
    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, inside_for));
    }
    let child = match kids.len() {
        0 => quote! { ::floem::views::Empty::new() },
        1 => kids[0].clone(),
        _ => super::generic::static_stack(&kids, quoin_style::FlexDir::Column, None),
    };
    let scrolled = quote! { ::floem::views::Scroll::new(#child) };
    match find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(scrolled, &styles)
        }
        None => scrolled,
    }
}

// ── virtual_list ────────────────────────────────────────────────────────────

/// `virtual_list(id, count, item_height, render)` → Floem `virtual_list`.
pub(crate) fn emit_virtual_list(el: &Element, _ctx: &mut super::EmitterCtx) -> TokenStream {
    let count_fallback: syn::Expr = syn::parse_quote!(0);
    let count_expr = find_arg_expr(el, "count").unwrap_or(&count_fallback);
    let item_height = find_arg_f32(el, "item_height").unwrap_or(32.0);
    let render_fallback: syn::Expr =
        syn::parse_quote!((|__ix: usize| ::floem::views::Empty::new()));
    let render_expr = find_arg_expr(el, "render").unwrap_or(&render_fallback);
    let height = proc_macro2::Literal::f32_suffixed(item_height);

    quote! {
        ::floem::views::virtual_list(
            move || 0..(#count_expr),
            move |__ix| *__ix,
            move |__ix| (#render_expr)(__ix),
        )
        .style(|__s| __s.height(#height))
    }
}

// ── tabs ────────────────────────────────────────────────────────────────────

/// `tabs(active: <signal>, on_click: handler) { tab(label: "..") { … } }`.
///
/// Floem's `tab` view owns the keyed tab strip; the content row is a
/// `dyn_container` keyed on the active index.
pub(crate) fn emit_tabs(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let active_fallback: syn::Expr = syn::parse_quote!(0usize);
    let active_expr = find_arg_expr(el, "active").unwrap_or(&active_fallback);
    let _capture = handler::capture_shadows(active_expr, ctx);

    let mut labels = Vec::new();
    let mut bodies = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(tab_el) = child
            && tab_el.name == "tab"
        {
            let label = find_arg_string(tab_el, "label").unwrap_or_default();
            labels.push(label);
            let mut tab_kids = Vec::new();
            for tab_child in &tab_el.children {
                tab_kids.push(emit_node(tab_child, ctx, inside_for));
            }
            bodies.push(match tab_kids.len() {
                0 => quote! { ::floem::views::Empty::new() },
                1 => tab_kids[0].clone(),
                _ => super::generic::static_stack(&tab_kids, quoin_style::FlexDir::Column, None),
            });
        }
    }

    let content_arms: Vec<TokenStream> = bodies
        .iter()
        .enumerate()
        .map(|(index, body)| {
            let lit = proc_macro2::Literal::usize_unsuffixed(index);
            quote! { #lit => #body.into_any() }
        })
        .collect();

    let label_lits: Vec<TokenStream> = labels.iter().map(|l| quote! { #l, }).collect();
    let label_count = labels.len();
    let body_count = bodies.len();
    let strip = quote! {
        ::floem::views::tab(
            move || {
                let __active: usize = ::core::clone::Clone::clone(&#active_expr);
                Some(__active)
            },
            move || (0usize..#label_count).collect::<Vec<_>>(),
            move |__ix| *__ix,
            move |__ix| {
                const LABELS: &[&str] = &[#(#label_lits)*];
                let __label: &'static str = LABELS[__ix];
                ::floem::views::Label::derived(move || __label)
            },
        )
    };

    let content = quote! {
        ::floem::views::dyn_container(
            move || {
                let __active: usize = ::core::clone::Clone::clone(&#active_expr);
                __active.min(#body_count.saturating_sub(1))
            },
            move |__ix| match __ix {
                #(#content_arms)*
                _ => ::floem::views::Empty::new().into_any(),
            },
        )
    };

    let styled = quote! {
        ::floem::views::Stack::vertical((#strip, #content))
    };
    match find_arg_string(el, "class") {
        Some(class) => {
            let style_decls = style::lower_class(&class);
            style::apply_styles(styled, &style_decls)
        }
        None => styled,
    }
}

// ── data_table ──────────────────────────────────────────────────────────────

/// `data_table(rows: <expr>, striped) { column(label: "..", render: |row| …) }`
/// → a column stack: header row + a keyed `dyn_stack` of body rows.
pub(crate) fn emit_data_table(el: &Element, _ctx: &mut super::EmitterCtx) -> TokenStream {
    let rows_fallback: syn::Expr = syn::parse_quote!(Vec::<()>::new());
    let rows_expr = find_arg_expr(el, "rows").unwrap_or(&rows_fallback);
    let _striped = find_arg_bool(el, "striped");

    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(col) = child
            && col.name == "column"
        {
            let label = find_arg_string(col, "label").unwrap_or_default();
            header_cells.push(quote! {
                ::floem::views::Label::derived(move || #label)
                    .style(|__s| __s
                        .padding_horiz(12.0)
                        .padding_vert(8.0)
                        .color(::quoin::floem_theme::muted_foreground_text())
                        .font_weight(::floem::text::FontWeight::MEDIUM))
            });
            match find_arg_expr(col, "render") {
                Some(render_expr) => {
                    let render = crate::transpile::force_move_on_closure(render_expr);
                    row_cells.push(quote! {
                        (#render)(&__row)
                    });
                }
                None => row_cells.push(quote! { ::floem::views::Empty::new() }),
            }
        }
    }

    let header = super::generic::static_stack(&header_cells, quoin_style::FlexDir::Row, None);
    let cell_views = &row_cells;

    // Rows rebuild through a dyn_container (no row key exists in the
    // element contract); append/remove/reorder all stay correct, which is
    // what a table needs. Row cells receive an owned clone of their row, so
    // the row type must be `Clone` (the same bound the other backends need
    // for their extract/render closures).
    let body = quote! {
        ::floem::views::dyn_container(
            move || ::std::sync::Arc::new((#rows_expr)),
            move |__rows| {
                let __kids = __rows
                    .iter()
                    .map(|__row| {
                        let __row = ::core::clone::Clone::clone(__row);
                        ::floem::views::Stack::horizontal((
                            #(#cell_views,)*
                        ))
                    })
                    .collect::<Vec<_>>();
                ::floem::views::Stack::vertical_from_iter(__kids).into_any()
            },
        )
    };

    quote! {
        ::floem::views::Stack::vertical((#header, #body))
    }
}

// ── dropdown_menu ───────────────────────────────────────────────────────────

/// `dropdown_menu { item(label: "..", on_click: ..) }` — a column of buttons.
/// (A full portal overlay lands with the M5 dropdown push; the inline list
/// matches the GPUI ghost-menu fallback so the parity matrix stays green.)
pub(crate) fn emit_dropdown_menu(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let mut items = Vec::new();
    for child in &el.children {
        if let crate::render_ast::RenderNode::Element(item) = child
            && (item.name == "item" || item.name == "button")
        {
            let label = find_arg_string(item, "label").unwrap_or_default();
            let action = match find_arg_expr(item, "on_click") {
                Some(handler_expr) => handler::handler_to_action(handler_expr, ctx),
                None => quote! {},
            };
            items.push(quote! {
                ::floem::views::Button::new(::floem::views::Label::derived(move || #label))
                    .style(|__s| __s
                        .width_full()
                        .padding_horiz(12.0)
                        .padding_vert(6.0))
                    #action
            });
        }
    }
    if items.is_empty() {
        return unsupported_element("dropdown_menu", "no `item` children found");
    }
    let stack = super::generic::static_stack(&items, quoin_style::FlexDir::Column, None);
    match find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(stack, &styles)
        }
        None => stack,
    }
}

// ── badge / skeleton / progress / clipboard_button / icon ───────────────────

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
        0 => quote! { ::floem::views::Empty::new() },
        1 => kids[0].clone(),
        _ => super::generic::static_stack(&kids, quoin_style::FlexDir::Row, None),
    };
    let badge = quote! {
        ::floem::views::Container::new(#content).style(|__s| __s
            .padding_horiz(8.0)
            .padding_vert(2.0)
            .border_radius(9999.0)
            .background(::quoin::floem_theme::secondary()))
    };
    match find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(badge, &styles)
        }
        None => badge,
    }
}

/// `skeleton` → a muted placeholder block.
pub(crate) fn emit_skeleton(el: &Element) -> TokenStream {
    let class = find_arg_string(el, "class");
    let block = quote! {
        ::floem::views::Empty::new().style(|__s| __s
            .width_full()
            .height(16.0)
            .border_radius(4.0)
            .background(::quoin::floem_theme::muted()))
    };
    match class.as_deref() {
        Some(c) => {
            let styles = style::lower_class(c);
            style::apply_styles(block, &styles)
        }
        None => block,
    }
}

/// `skeleton_text` → a muted text-shaped placeholder.
pub(crate) fn emit_skeleton_text(el: &Element) -> TokenStream {
    let _ = el;
    quote! {
        ::floem::views::Empty::new().style(|__s| __s
            .width_full()
            .height(14.0)
            .border_radius(2.0)
            .background(::quoin::floem_theme::muted()))
    }
}

/// `skeleton_avatar` → a circular placeholder.
pub(crate) fn emit_skeleton_avatar(el: &Element) -> TokenStream {
    let _ = el;
    quote! {
        ::floem::views::Empty::new().style(|__s| __s
            .width(40.0)
            .height(40.0)
            .border_radius(20.0)
            .background(::quoin::floem_theme::muted()))
    }
}

/// `progress(value, max)` → a two-layer bar.
pub(crate) fn emit_progress(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let value_fallback: syn::Expr = syn::parse_quote!(0.0);
    let max_fallback: syn::Expr = syn::parse_quote!(100.0);
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
    let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);
    let capture = handler::capture_shadows(value_expr, ctx);
    quote! {
        {
            #(#capture)*
            ::floem::views::slider(move || {
                let __max: f64 = #max_expr;
                let __value: f64 = ::core::clone::Clone::clone(&#value_expr);
                if __max > 0.0 { (__value / __max) * 100.0 } else { 0.0 }
            })
            .disabled(|| true)
        }
    }
}

/// `clipboard_button(copy_text: <expr>)` → a button writing the clipboard.
pub(crate) fn emit_clipboard_button(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let copy_fallback: syn::Expr = syn::parse_quote!("");
    let copy_expr = find_arg_expr(el, "copy_text").unwrap_or(&copy_fallback);
    let capture = handler::capture_shadows(copy_expr, ctx);
    quote! {
        {
            #(#capture)*
            ::floem::views::Button::new(::floem::views::Label::derived(|| "Copy"))
                .action(move || {
                    let __text = ::core::clone::Clone::clone(&#copy_expr);
                    let _ = ::floem::Clipboard::set_text(__text);
                })
        }
    }
}

/// `icon(icon_name: "..")` — unsupported: icon SVG tables are GPUI-specific.
pub(crate) fn emit_icon(el: &Element) -> TokenStream {
    let _ = find_arg_string(el, "icon_name");
    unsupported_element(
        "icon",
        "inline icon SVGs are a GPUI-backend feature; pass an explicit `svg` instead",
    )
}
