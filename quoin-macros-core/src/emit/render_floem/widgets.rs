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

/// `separator` → shadcn `Separator` (feature `floem-shadcn`) or the 1px
/// `border`-token rule (generic v1 contract).
pub(crate) fn emit_separator(el: &Element) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let separator = quote! { ::floem_shadcn::components::separator::Separator::new() };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(separator, &styles)
            }
            None => separator,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
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
    #[cfg(feature = "floem-shadcn")]
    {
        let checked_fallback: syn::Expr =
            syn::parse_quote!((|| unreachable!("checkbox/switch requires `checked:`")));
        let checked_expr = find_arg_expr(el, "checked").unwrap_or(&checked_fallback);
        let capture = handler::capture_shadows(checked_expr, ctx);

        // The accessible label: `label:` argument or a single text child.
        let label_chain = match find_arg_string(el, "label") {
            Some(text) => quote! { .label(#text) },
            None => {
                let mut texts = Vec::new();
                for child in &el.children {
                    if let crate::render_ast::RenderNode::Text(lit) = child {
                        texts.push(lit.clone());
                    }
                }
                match texts.as_slice() {
                    [lit] => quote! { .label(#lit) },
                    _ => quote! {},
                }
            }
        };

        let change_hook =
            match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_checked_change"))
            {
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

        let component = if switch {
            quote! { ::floem_shadcn::components::switch::Switch::new(__state) }
        } else {
            quote! { ::floem_shadcn::components::checkbox::Checkbox::new(__state) }
        };

        let disabled_lit = find_arg_bool(el, "disabled");

        // Same two-way guarded bridge as the generic path; the shadcn
        // component owns the click toggle natively over `__state`.
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
                #component #label_chain .disabled(#disabled_lit)
            }
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        let checked_fallback: syn::Expr =
            syn::parse_quote!((|| unreachable!("checkbox/switch requires `checked:`")));
        let checked_expr = find_arg_expr(el, "checked").unwrap_or(&checked_fallback);
        let capture = handler::capture_shadows(checked_expr, ctx);

        let change_hook =
            match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_checked_change"))
            {
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
}

// ── slider ──────────────────────────────────────────────────────────────────

/// `slider(value, min, max, on_change)` → shadcn `Slider` (feature
/// `floem-shadcn`) or Floem `slider` + `SliderChanged` (generic).
pub(crate) fn emit_slider(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let value_fallback: syn::Expr = syn::parse_quote!(0.0);
        let min_fallback: syn::Expr = syn::parse_quote!(0.0);
        let max_fallback: syn::Expr = syn::parse_quote!(100.0);
        let step_fallback: syn::Expr = syn::parse_quote!(1.0);
        let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
        let min_expr = find_arg_expr(el, "min").unwrap_or(&min_fallback);
        let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);
        let step_expr = find_arg_expr(el, "step").unwrap_or(&step_fallback);

        let capture = handler::capture_shadows(value_expr, ctx);

        let change_hook = match find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"))
        {
            Some(handler_expr) => {
                let (block, _name) = handler::handler_block(handler_expr, ctx);
                let call = if handler::handler_arity_is_zero(handler_expr) {
                    quote! { __h() }
                } else {
                    quote! { __h(__current) }
                };
                quote! {
                    #block
                    ::floem::reactive::Effect::new(
                        move |__prev: ::core::option::Option<f64>| {
                            let __current = ::floem::reactive::SignalGet::get(&__state);
                            if __prev != Some(__current) {
                                #call;
                            }
                            __current
                        },
                    );
                }
            }
            None => quote! {},
        };

        let disabled_lit = find_arg_bool(el, "disabled");

        // Same guarded two-way bridge as checkbox/switch, over `f64`.
        quote! {
            {
                #(#capture)*
                let __value = ::core::clone::Clone::clone(&(#value_expr));
                let __state = ::floem::reactive::RwSignal::new(
                    ::quoin::Signal::get(&__value) as f64
                );
                ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                    let _ = __prev;
                    let __next = ::quoin::Signal::get(&__value) as f64;
                    if ::floem::reactive::SignalGet::get_untracked(&__state) != __next {
                        ::floem::reactive::SignalUpdate::set(&__state, __next);
                    }
                });
                ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                    let _ = __prev;
                    let __edited = ::floem::reactive::SignalGet::get(&__state);
                    if ::quoin::Signal::get(&__value) as f64 != __edited {
                        ::quoin::Signal::set(&__value, __edited);
                    }
                });
                #change_hook
                let __min: f64 = #min_expr;
                let __max: f64 = #max_expr;
                let __step: f64 = #step_expr;
                ::floem_shadcn::components::slider::Slider::new(__state)
                    .min(__min)
                    .max(__max)
                    .step(__step)
                    .disabled(#disabled_lit)
            }
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
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

/// `virtual_list(items, estimated_height, render)` → Floem `virtual_list`.
///
/// The parser enforces the shared contract (`items:` + `estimated_height:`);
/// the range is the items' length.
pub(crate) fn emit_virtual_list(el: &Element, _ctx: &mut super::EmitterCtx) -> TokenStream {
    let items_fallback: syn::Expr = syn::parse_quote!(Vec::<()>::new());
    let items_expr = find_arg_expr(el, "items").unwrap_or(&items_fallback);
    let item_height = find_arg_f32(el, "estimated_height").unwrap_or(32.0);
    let render_fallback: syn::Expr =
        syn::parse_quote!((|__ix: usize| ::floem::views::Empty::new()));
    let render_expr = find_arg_expr(el, "render").unwrap_or(&render_fallback);
    let height = proc_macro2::Literal::f32_suffixed(item_height);

    quote! {
        ::floem::views::virtual_list(
            move || 0..(#items_expr).len(),
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
            quote! { #lit => #body.into_any(), }
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

    #[cfg(feature = "floem-shadcn")]
    {
        let choice = crate::emit::shadcn_spec::canonical_badge_variant(el);
        if let Some(error) =
            choice.as_ref().and_then(crate::emit::shadcn_spec::choice_error)
        {
            return error;
        }
        let variant_chain = match choice.and_then(|c| c.ok()) {
            Some("secondary") => {
                quote! { .with_variant(::floem_shadcn::components::badge::BadgeVariant::Secondary) }
            }
            Some("destructive") => {
                quote! { .with_variant(::floem_shadcn::components::badge::BadgeVariant::Destructive) }
            }
            Some("outline") => {
                quote! { .with_variant(::floem_shadcn::components::badge::BadgeVariant::Outline) }
            }
            _ => quote! {},
        };
        let badge = quote! {
            ::floem_shadcn::components::badge::Badge::new(#content) #variant_chain
        };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(badge, &styles)
            }
            None => badge,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
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
}

/// `skeleton` → a muted placeholder block.
pub(crate) fn emit_skeleton(el: &Element) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        // The shadcn skeleton is an empty shimmering block: width/height
        // come from args, not children.
        if !el.children.is_empty() {
            return quote! {
                ::core::compile_error!(
                    "`skeleton` takes `width:`/`height:` (f32) arguments, not children"
                )
            };
        }
        let width = proc_macro2::Literal::f64_suffixed(find_arg_f32(el, "width").unwrap_or(96.0) as f64);
        let height =
            proc_macro2::Literal::f64_suffixed(find_arg_f32(el, "height").unwrap_or(16.0) as f64);
        let block = quote! {
            ::floem_shadcn::components::skeleton::Skeleton::new()
                .width(#width)
                .height(#height)
        };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(block, &styles)
            }
            None => block,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
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
    #[cfg(feature = "floem-shadcn")]
    {
        let value_fallback: syn::Expr = syn::parse_quote!(0.0);
        let max_fallback: syn::Expr = syn::parse_quote!(100.0);
        let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);
        let max_expr = find_arg_expr(el, "max").unwrap_or(&max_fallback);
        let capture = handler::capture_shadows(value_expr, ctx);

        // Read-only display: one-way quoin → floem sync (no write-back).
        quote! {
            {
                #(#capture)*
                let __value = ::core::clone::Clone::clone(&(#value_expr));
                let __state = ::floem::reactive::RwSignal::new(
                    ::quoin::Signal::get(&__value) as f64
                );
                ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                    let _ = __prev;
                    let __next = ::quoin::Signal::get(&__value) as f64;
                    if ::floem::reactive::SignalGet::get_untracked(&__state) != __next {
                        ::floem::reactive::SignalUpdate::set(&__state, __next);
                    }
                });
                let __max: f64 = #max_expr;
                ::floem_shadcn::components::progress::Progress::new(__state).max(__max)
            }
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
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

// ── rich-elements Phase 1: canonical shadcn Tier-1 surface ─────────────────
//
// Each element has two lowerings switched by the `floem-shadcn` feature:
// the shadcn component (native look, thread-local `ShadcnTheme`) or the
// generic container fallback (same category as `div` — always compiles).
// The per-framework backing map lives in `emit::shadcn_spec`.

/// `alert(title:, description:, variant:|destructive:)` → shadcn `Alert`.
///
/// The builder takes the copy as arguments; element children are rejected
/// with guidance (`alert_title`/`alert_description` stay generic surface
/// elements for the web-style backends).
pub(crate) fn emit_alert(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let _ = (ctx, inside_for);
        if !el.children.is_empty() {
            return quote! {
                ::core::compile_error!(
                    "`alert` takes `title:`/`description:` arguments; \
                     children are not part of the Floem shadcn contract"
                )
            };
        }
        let choice = crate::emit::shadcn_spec::canonical_alert_variant(el);
        if let Some(error) =
            choice.as_ref().and_then(crate::emit::shadcn_spec::choice_error)
        {
            return error;
        }
        let variant_chain = match choice.and_then(|c| c.ok()) {
            Some("destructive") => quote! { .destructive() },
            _ => quote! {},
        };
        let title_chain = match find_arg_string(el, "title") {
            Some(title) => quote! { .title(#title) },
            None => quote! {},
        };
        let description_chain = match find_arg_string(el, "description") {
            Some(description) => quote! { .description(#description) },
            None => quote! {},
        };
        let alert = quote! {
            ::floem_shadcn::components::alert::Alert::new()
                #variant_chain
                #title_chain
                #description_chain
        };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(alert, &styles)
            }
            None => alert,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `card { … }` → shadcn `Card` over the static child tuple.
pub(crate) fn emit_card(el: &Element, ctx: &mut super::EmitterCtx, inside_for: bool) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let mut kids = Vec::new();
        for child in &el.children {
            kids.push(emit_node(child, ctx, inside_for));
        }
        let body = match kids.len() {
            0 => {
                return quote! {
                    ::core::compile_error!("`card` requires children (card_header/card_content/…)")
                };
            }
            1 => {
                let only = &kids[0];
                quote! { (#only,) }
            }
            _ => quote! { (#(#kids,)*) },
        };
        let card = quote! { ::floem_shadcn::components::card::Card::new(#body) };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(card, &styles)
            }
            None => card,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `card_header(title:, description:)` → shadcn `CardHeader`.
///
/// The builder carries the copy as arguments; child elements are rejected
/// with guidance (use `card_title`/`card_description` as direct `card`
/// children if a standalone header part is needed).
pub(crate) fn emit_card_header(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let _ = (ctx, inside_for);
        if !el.children.is_empty() {
            return quote! {
                ::core::compile_error!(
                    "`card_header` takes `title:`/`description:` arguments; \
                     children are not part of the Floem shadcn contract"
                )
            };
        }
        let title_chain = match find_arg_string(el, "title") {
            Some(title) => quote! { .title(#title) },
            None => quote! {},
        };
        let description_chain = match find_arg_string(el, "description") {
            Some(description) => quote! { .description(#description) },
            None => quote! {},
        };
        let header = quote! {
            ::floem_shadcn::components::card::CardHeader::new() #title_chain #description_chain
        };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(header, &styles)
            }
            None => header,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `card_title`/`card_description`/`card_content`/`card_footer`/`card_action`
/// → the matching shadcn part over the (stacked) child views.
pub(crate) fn emit_card_part(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let part = el.name.to_string();
        let mut kids = Vec::new();
        for child in &el.children {
            kids.push(emit_node(child, ctx, inside_for));
        }
        let child: TokenStream = match kids.len() {
            0 => quote! { ::floem::views::Empty::new() },
            1 => kids[0].clone(),
            _ => super::generic::static_stack(&kids, quoin_style::FlexDir::Column, None),
        };
        let ctor = match part.as_str() {
            "card_title" => quote! { ::floem_shadcn::components::card::CardTitle::new },
            "card_description" => quote! { ::floem_shadcn::components::card::CardDescription::new },
            "card_content" => quote! { ::floem_shadcn::components::card::CardContent::new },
            "card_footer" => quote! { ::floem_shadcn::components::card::CardFooter::new },
            "card_action" => quote! { ::floem_shadcn::components::card::CardAction::new },
            _ => unreachable!("emit_card_part only dispatches card parts"),
        };
        let view = quote! { #ctor(#child) };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(view, &styles)
            }
            None => view,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `spinner` → shadcn `Spinner` (default size/color).
pub(crate) fn emit_spinner(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let _ = (ctx, inside_for);
        let spinner = quote! { ::floem_shadcn::components::spinner::Spinner::new() };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(spinner, &styles)
            }
            None => spinner,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `kbd(text: "..")` (or a single text child) → shadcn `Kbd` key cap.
pub(crate) fn emit_kbd(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let _ = (ctx, inside_for);
        let text = match find_arg_string(el, "text").or_else(|| find_arg_string(el, "label")) {
            Some(text) => Some(text),
            None => {
                let mut texts = Vec::new();
                for child in &el.children {
                    if let crate::render_ast::RenderNode::Text(lit) = child {
                        texts.push(lit.clone());
                    }
                }
                match texts.as_slice() {
                    [lit] => Some(lit.value()),
                    _ => None,
                }
            }
        };
        let Some(text) = text else {
            return quote! {
                ::core::compile_error!("`kbd` requires `text:` (or exactly one text child)")
            };
        };
        let kbd = quote! { ::floem_shadcn::components::kbd::Kbd::new(#text) };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(kbd, &styles)
            }
            None => kbd,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}

/// `avatar(fallback: "..", size:)` → shadcn `Avatar` (fallback initials).
pub(crate) fn emit_avatar(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        let _ = (ctx, inside_for);
        let fallback_chain = match find_arg_string(el, "fallback") {
            Some(fallback) => quote! { .fallback(#fallback) },
            None => quote! {},
        };
        let size_chain = match find_arg_f32(el, "size") {
            Some(size) => {
                let size = proc_macro2::Literal::f64_suffixed(size as f64);
                quote! { .size(#size) }
            }
            None => quote! {},
        };
        let avatar = quote! {
            ::floem_shadcn::components::avatar::Avatar::new() #fallback_chain #size_chain
        };
        match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(avatar, &styles)
            }
            None => avatar,
        }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        super::generic::emit_container(el, ctx, inside_for)
    }
}
