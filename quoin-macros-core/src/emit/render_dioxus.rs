#![allow(unused_variables)]
use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{
    collect_handler_idents, collect_handler_idents_excluding_params, force_move_on_closure,
};
use proc_macro2::TokenStream;
use quote::quote;

// ---------------------------------------------------------------------------
// Top‑level render entry point — wraps everything in rsx! {} and imports prelude
// ---------------------------------------------------------------------------
pub fn emit_render(node: &RenderNode) -> TokenStream {
    let inner = emit_render_inner(node);
    // CRITICAL: Bring dioxus::prelude into scope so rsx! can resolve dioxus_elements
    let tokens = quote! {
        {
            use dioxus::prelude::*;
            dioxus::prelude::rsx! { #inner }
        }
    };
    wrap_with_cfg(node_attrs(node), tokens)
}

fn node_attrs(node: &RenderNode) -> &[syn::Attribute] {
    match node {
        RenderNode::Element(el) => &el.attrs,
        RenderNode::If(if_node) => &if_node.attrs,
        RenderNode::For(for_node) => &for_node.attrs,
        RenderNode::Text(_) | RenderNode::Expr(_) | RenderNode::Root(_) => &[],
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
            let tokens: Vec<TokenStream> = nodes.iter().map(emit_render_inner).collect();
            quote! { #(#tokens)* }
        }
    }
}

// ---------------------------------------------------------------------------
// Dioxus‑specific handler wrapping (clone captures, force move)
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Element dispatch — applies cfg attributes at the element level
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Separator (Tier 1 — no variant, just tag swap)
// ---------------------------------------------------------------------------

fn resolve_separator_tag(el: &Element) -> TokenStream {
    let orientation =
        find_arg_string(el, "orientation").unwrap_or_else(|| "horizontal".to_string());
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::separator::Separator }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let tag = if orientation == "horizontal" {
            "hr"
        } else {
            "div"
        };
        let ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
        quote! { #ident }
    }
}

fn emit_separator(el: &Element) -> TokenStream {
    let tag = resolve_separator_tag(el);
    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: #s, });
                } else {
                    attrs.push(quote! { class: {#value}, });
                }
            }
            "orientation" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        quote! { #tag { } }
    } else {
        quote! { #tag { #(#attrs)* } }
    }
}

// ---------------------------------------------------------------------------
// Skeleton / SkeletonText / SkeletonAvatar (Tier 1 — no variant, just tag swap)
// ---------------------------------------------------------------------------

fn emit_skeleton(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse rounded-md bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse rounded-md bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

fn emit_skeleton_text(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse h-4 w-full rounded-md bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse h-4 w-full rounded-md bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

fn emit_skeleton_avatar(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse h-10 w-10 rounded-full bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse h-10 w-10 rounded-full bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

// ---------------------------------------------------------------------------
// Progress (Tier 2 — determinate value vs indeterminate)
// ---------------------------------------------------------------------------

fn emit_progress(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let max_expr = find_arg_expr(el, "max");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    let outer_cls = if user_class.is_empty() {
        "relative h-4 w-full overflow-hidden rounded-full bg-secondary"
    } else {
        // Dioxus class takes &str, interpolate
        ""
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let value_prop = match value_expr {
            Some(val) => match max_expr {
                Some(m) => quote! { value={format!("{}", (#val as f64) / (#m as f64))} },
                None => quote! { value={format!("{}", (#val as f64) / 100.0)} },
            },
            None => quote! {},
        };
        quote! { shadcn_dioxus::progress::Progress { #value_prop } }
    }

    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let bar_cls = "h-full rounded-full bg-primary transition-all duration-300";
        match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) * 100.0 },
                    None => quote! { (#val as f64) },
                };
                let full_cls = if user_class.is_empty() {
                    outer_cls
                } else {
                    &user_class
                };
                quote! {
                    div { class: #full_cls,
                        div { class: #bar_cls, style: "width: {#max}%" }
                    }
                }
            }
            None => {
                let indeterminate_cls =
                    "h-full w-1/3 rounded-full bg-primary animate-indeterminate";
                let full_cls = if user_class.is_empty() {
                    outer_cls
                } else {
                    &user_class
                };
                quote! {
                    div { class: #full_cls,
                        div { class: #indeterminate_cls }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Checkbox (Tier 2 — type=checkbox vs shadcn Checkbox)
// ---------------------------------------------------------------------------

fn emit_checkbox(el: &Element) -> TokenStream {
    let checked_expr = find_arg_expr(el, "on_checked_change")
        .or_else(|| find_arg_expr(el, "on_change"))
        .is_some()
        .then(|| find_arg_expr(el, "checked"))
        .flatten();
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    let base = "h-4 w-4 rounded border border-input ring-offset-background accent-primary-500 cursor-pointer";
    let full_class = if user_class.is_empty() {
        base
    } else {
        &user_class
    };

    let checked_attr = match checked_expr {
        Some(val) => quote! { checked: #val, },
        None => quote! {},
    };

    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { onchange: #wrapped, }
        }
        None => quote! {},
    };

    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::checkbox::Checkbox { #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "checkbox", class: #full_class, #checked_attr #on_change_attr #disabled_attr } }
    }
}

// ---------------------------------------------------------------------------
// Switch (Tier 2 — toggle-switch styled checkbox)
// ---------------------------------------------------------------------------

fn emit_switch(el: &Element) -> TokenStream {
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");

    let checked_attr = match checked_expr {
        Some(val) => quote! { checked: #val, },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { onchange: #wrapped, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::switch::Switch { #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let track_cls = "relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors bg-input";
        let thumb_cls = "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform translate-x-0";
        quote! {
            button {
                class: #track_cls,
                role: "switch",
                #disabled_attr
                onclick: move |_| {},
                input { r#type: "checkbox", #checked_attr #on_change_attr, class: "sr-only peer" },
                div { class: #thumb_cls }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RadioGroup / Radio (Tier 2 - grouped radio inputs)
// ---------------------------------------------------------------------------

fn emit_radio_group(el: &Element) -> TokenStream {
    let children: Vec<TokenStream> = el.children.iter().map(|c| emit_render_inner(c)).collect();
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::radio_group::RadioGroup { class: #user_class, #(#children)* } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let cls = if user_class.is_empty() {
            "flex flex-col gap-2"
        } else {
            &user_class
        };
        quote! { div { class: #cls, #(#children)* } }
    }
}

fn emit_radio(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let name_expr = find_arg_expr(el, "name");
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr = find_arg_expr(el, "on_change");
    let disabled = find_arg_bool(el, "disabled");

    let base = "h-4 w-4 rounded-full border border-input accent-primary-500 cursor-pointer";
    let checked_attr = match checked_expr {
        Some(v) => quote! { checked: #v, },
        None => quote! {},
    };
    let name_attr = match name_expr {
        Some(n) => quote! { name: #n, },
        None => quote! {},
    };
    let value_attr = match value_expr {
        Some(v) => quote! { value: #v },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let w = wrap_dioxus_handler(handler);
            quote! { onchange: #w, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::radio_group::RadioGroupItem { #value_attr #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "radio", class: #base, #checked_attr #name_attr #value_attr #on_change_attr #disabled_attr } }
    }
}

// ---------------------------------------------------------------------------
// Slider (Tier 2 - range input)
// ---------------------------------------------------------------------------

fn emit_slider(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let min_expr = find_arg_expr(el, "min");
    let max_expr = find_arg_expr(el, "max");
    let step_expr = find_arg_expr(el, "step");
    let on_change_expr = find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"));
    let disabled = find_arg_bool(el, "disabled");

    let base =
        "w-full h-2 rounded-lg appearance-none cursor-pointer accent-primary-500 bg-transparent";

    let value_attr = match value_expr {
        Some(v) => quote! { value: #v, },
        None => quote! {},
    };
    let min_attr = match min_expr {
        Some(m) => quote! { min: #m, },
        None => quote! {},
    };
    let max_attr = match max_expr {
        Some(m) => quote! { max: #m, },
        None => quote! {},
    };
    let step_attr = match step_expr {
        Some(s) => quote! { step: #s, },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let w = wrap_dioxus_handler(handler);
            quote! { onchange: #w, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::slider::Slider { #value_attr #min_attr #max_attr #step_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "range", class: #base, #value_attr #min_attr #max_attr #step_attr #on_change_attr #disabled_attr } }
    }
}

// ---------------------------------------------------------------------------
// Tooltip (Tier 2 - standalone tooltip element)
// ---------------------------------------------------------------------------

fn emit_tooltip(el: &Element) -> TokenStream {
    let trigger_expr = &el.trigger_expr;
    let text = find_arg_string(el, "text").unwrap_or_default();

    // Case 1: No trigger — simple title-attribute tooltip
    if trigger_expr.is_none() {
        return quote! { span { title: #text } {#text} };
    }

    // Case 2: With trigger — wrap trigger in hover tooltip wrapper
    let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone().unwrap()));

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! {
            shadcn_dioxus::tooltip::Tooltip {
                shadcn_dioxus::tooltip::TooltipTrigger { #trigger_inner }
                shadcn_dioxus::tooltip::TooltipContent { #text }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! {
            {
                let mut __tip_open = dioxus::prelude::use_signal(|| false);
                dioxus::prelude::rsx! {
                    div {
                        class: "relative inline-block",
                        onmouseenter: move |_| __tip_open.set(true),
                        onmouseleave: move |_| __tip_open.set(false),
                        #trigger_inner,
                        if *__tip_open.read() {
                            div {
                                class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                #text
                            }
                        }
                    }
                }
            }
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
        "separator" => emit_separator(el),
        "skeleton" => emit_skeleton(el),
        "skeleton_text" => emit_skeleton_text(el),
        "skeleton_avatar" => emit_skeleton_avatar(el),
        "progress" => emit_progress(el),
        "checkbox" => emit_checkbox(el),
        "switch" => emit_switch(el),
        "radio_group" => emit_radio_group(el),
        "radio" => emit_radio(el),
        "slider" => emit_slider(el),
        "tooltip" => emit_tooltip(el),
        "tabs" => emit_tabs(el),
        "data_table" => emit_data_table(el),
        "dropdown_menu" => emit_dropdown_menu(el),
        "styled_text" => emit_styled_text(el),
        "badge" => emit_badge(el),
        "scroll_area" => emit_scroll_area(el),
        "virtual_list" => emit_virtual_list(el),
        "clipboard_button" => emit_clipboard_button(el),
        "button" => emit_button(el),
        "icon" => emit_icon(el),
        "input" => emit_input(el),
        _ => emit_html_el(el, &name_str),
    }
}

// ---------------------------------------------------------------------------
// Virtual list
//
// WARNING: This is a stub implementation that does NOT provide true virtualization.
// All child elements are rendered into a scrollable container regardless of the
// number of items. The `estimated_height` parameter only sets the container's
// fixed height via CSS but does NOT affect which items are rendered.
//
// For large lists (1000+ items), this will have significant performance overhead
// compared to a proper virtualized implementation. True virtualization (only
// rendering visible items based on scroll position) is not yet implemented.
//
// If you need true virtualization for large datasets, consider using a framework-
// specific virtualization library directly instead of this component.
// ---------------------------------------------------------------------------
fn emit_virtual_list(el: &Element) -> TokenStream {
    let estimated_height = find_arg_f32(el, "estimated_height");
    let children_tokens: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    let style = match estimated_height {
        Some(h) => format!("overflow-y: auto; height: {}px", h),
        None => "overflow-y: auto".to_string(),
    };
    quote! { div { style: #style, #(#children_tokens)* } }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------
fn emit_badge(el: &Element) -> TokenStream {
    // --- Shared computation (always runs) ---
    let color_expr = find_arg_expr(el, "color");
    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

    // --- Branching: tag + render structure ---
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let class_str = if let Some(color) = color_expr {
            let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
            match bg_class {
                Some(cls) => format!(
                    "inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}",
                    cls
                ),
                None => {
                    "inline-flex items-center px-1.5 rounded text-xs font-medium text-white bg-gray-600"
                        .to_string()
                }
            }
        } else {
            "inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white"
                .to_string()
        };

        if children.is_empty() {
            quote! { shadcn_dioxus::badge::Badge { class: #class_str } }
        } else {
            quote! { shadcn_dioxus::badge::Badge { class: #class_str, #(#children)* } }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        match color_expr {
            Some(color) => {
                let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
                match bg_class {
                    Some(cls) => quote! {
                        span {
                            class: format!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}", #cls),
                            #(#children)*
                        }
                    },
                    None => quote! {
                        span {
                            class: "inline-flex items-center px-1.5 rounded text-xs font-medium text-white bg-gray-600",
                            #(#children)*
                        }
                    },
                }
            }
            None => quote! {
                span { class: "inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white", #(#children)* }
            },
        }
    }
}
// ---------------------------------------------------------------------------
// Scroll area
// ---------------------------------------------------------------------------
fn emit_scroll_area(el: &Element) -> TokenStream {
    let direction = find_arg_string(el, "direction").unwrap_or_else(|| "vertical".to_string());

    let overflow_class = match direction.as_str() {
        "horizontal" => "overflow-x-auto",
        "both" => "overflow-auto",
        _ => "overflow-y-auto",
    };

    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: format!("{} {}", #s, #overflow_class), });
                } else {
                    attrs.push(quote! { class: format!("{} {}", #value, #overflow_class), });
                }
            }
            "direction" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        attrs.push(quote! { class: #overflow_class, });
    }
    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    quote! { div { #(#attrs)* #(#children)* } }
}

// ---------------------------------------------------------------------------
// Button (with optional tooltip)
// ---------------------------------------------------------------------------
fn emit_button(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");
        let class_expr = find_arg_expr(el, "class");

        let primary = find_arg_bool(el, "primary");
        let destructive = find_arg_bool(el, "destructive");
        let ghost = find_arg_bool(el, "ghost");
        let disabled = find_arg_bool(el, "disabled");

        let variant_class = if destructive {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        } else if ghost {
            "hover:bg-accent hover:text-accent-foreground"
        } else if primary {
            "bg-primary text-primary-foreground hover:bg-primary/90"
        } else {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        };

        let base_class = "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors cursor-pointer";
        let disabled_class = if disabled {
            " opacity-50 pointer-events-none"
        } else {
            ""
        };

        let full_class = match class_expr {
            Some(cls) => {
                quote! { format!("{} {}{} {}", #base_class, #variant_class, #disabled_class, #cls) }
            }
            None => quote! { format!("{} {}{}", #base_class, #variant_class, #disabled_class) },
        };

        let on_click_attr = find_arg_expr(el, "on_click").map(|handler_expr| {
            let handler = wrap_dioxus_handler(handler_expr);
            quote! { onclick: {#handler}, }
        });

        let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

        let inner_button = if children.is_empty() {
            quote! {
                button { class: #full_class, #on_click_attr }
            }
        } else {
            quote! {
                button { class: #full_class, #on_click_attr #(#children)* }
            }
        };

        match tooltip_text {
            Some(text) => quote! {
                {
                    let mut __tip_open = dioxus::prelude::use_signal(|| false);
                    dioxus::prelude::rsx! {
                        div {
                            class: "relative inline-flex",
                            onmouseenter: move |_| __tip_open.set(true),
                            onmouseleave: move |_| __tip_open.set(false),
                            #inner_button,
                            if *__tip_open.read() {
                                div {
                                    class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                    #text
                                }
                            }
                        }
                    }
                }
            },
            None => inner_button,
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let tooltip_text = find_arg_string(el, "tooltip");

        let inner_button = emit_html_el_inner(el, "button");

        match tooltip_text {
            Some(text) => quote! {
                {
                    let mut __tip_open = dioxus::prelude::use_signal(|| false);
                    dioxus::prelude::rsx! {
                        div {
                            class: "relative inline-flex",
                            onmouseenter: move |_| __tip_open.set(true),
                            onmouseleave: move |_| __tip_open.set(false),
                            #inner_button,
                            if *__tip_open.read() {
                                div {
                                    class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50",
                                    #text
                                }
                            }
                        }
                    }
                }
            },
            None => inner_button,
        }
    }
}
// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------
fn emit_input(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();

        let class_expr = find_arg_expr(el, "class");
        let value_expr = find_arg_expr(el, "value");
        let on_input_expr = find_arg_expr(el, "on_input");
        let disabled = find_arg_bool(el, "disabled");

        let base_class = "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

        let full_class = match class_expr {
            Some(cls) => quote! { format!("{} {}", #base_class, #cls) },
            None => quote! { #base_class },
        };

        let placeholder_attr = if placeholder.is_empty() {
            quote! {}
        } else {
            quote! { placeholder: #placeholder, }
        };

        let disabled_attr = if disabled {
            quote! { disabled: true, }
        } else {
            quote! {}
        };

        // FIX: Use signal.get() directly, not string interpolation
        let value_attr = if let Some(val) = value_expr {
            quote! { value: {#val.get()}, }
        } else {
            quote! {}
        };

        let oninput_attr = if let Some(handler) = on_input_expr {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { oninput: #wrapped, }
        } else if let Some(val) = value_expr {
            quote! {
                oninput: move |ev: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                    #val.set(ev.value());
                },
            }
        } else {
            quote! {}
        };

        quote! {
            input {
                class: #full_class,
                #placeholder_attr
                #value_attr
                #oninput_attr
                #disabled_attr
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_html_el_inner(el, "input")
    }
}
// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------
fn emit_icon(el: &Element) -> TokenStream {
    let name = find_arg_string(el, "icon_name");
    let size_class = find_arg_expr(el, "class");

    let class_str = match size_class {
        Some(c) => quote! { format!("{} w-4 h-4 inline-block", #c) },
        None => quote! { "w-4 h-4 inline-block" },
    };

    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

    match name {
        Some(n) => {
            if let Some(svg) = crate::transpile::icon_codegen::icon_svg_html(&n) {
                quote! { span { class: #class_str, #svg } }
            } else {
                quote! { span { class: #class_str, "\u{2753}" } }
            }
        }
        None => {
            if children.is_empty() {
                quote! { span { class: #class_str, "\u{2753}" } }
            } else {
                quote! { span { class: #class_str, #(#children)* } }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// StyledText
// ---------------------------------------------------------------------------
fn emit_styled_text(el: &Element) -> TokenStream {
    let text_expr = find_arg_expr(el, "text");
    let query_expr = find_arg_expr(el, "query");

    match (text_expr, query_expr) {
        (Some(text), None) => {
            quote! { span { #text } }
        }
        (Some(text), Some(query)) => {
            quote! {
                {
                    fn __quoin_styled_text(__text_val: String, __query_val: String) -> dioxus::prelude::Element {
                        let mut __parts: Vec<dioxus::prelude::Element> = Vec::new();
                        if !__query_val.is_empty() {
                            let mut __remaining: &str = &__text_val;
                            let __query_lower: String = __query_val.to_lowercase();
                            while let Some(__idx) = __remaining.to_lowercase().find(&__query_lower) {
                                if __idx > 0 {
                                    let __before: String = __remaining[..__idx].to_string();
                                    __parts.push(dioxus::prelude::rsx! { span { "{__before}" } });
                                }
                                let __match_str: String = __remaining[__idx..__idx + __query_val.len()].to_string();
                                __parts.push(dioxus::prelude::rsx! {
                                    span { class: "bg-yellow-200 text-black", "{__match_str}" }
                                });
                                __remaining = &__remaining[__idx + __query_val.len()..];
                            }
                        }
                        if __parts.is_empty() {
                            __parts.push(dioxus::prelude::rsx! { span { "{__text_val}" } });
                        }
                        dioxus::prelude::rsx! { span { {__parts.into_iter()} } }
                    }
                    __quoin_styled_text((#text).clone(), (#query).clone())
                }
            }
        }
        (None, _) => {
            quote! { span {} }
        }
    }
}

// ---------------------------------------------------------------------------
// Clipboard button
// ---------------------------------------------------------------------------
fn emit_clipboard_button(el: &Element) -> TokenStream {
    let copy_text = match find_arg_expr(el, "copy_text") {
        Some(ct) => ct,
        None => return emit_html_el(el, "button"),
    };

    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: #s, });
                } else {
                    attrs.push(quote! { class: {#value}, });
                }
            }
            "disabled" => attrs.push(quote! { disabled: #value, }),
            "copy_text" => {}
            _ => {}
        }
    }

    let mut children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();
    if children.is_empty() {
        children.push(quote! { "Copy" });
    }

    quote! {
        button {
            #(#attrs)*
            onclick: move |_| {
                quoin::clipboard_write_text(&(#copy_text).to_string());
            },
            #(#children)*
        }
    }
}

// ---------------------------------------------------------------------------
// HTML element emitter — core RSX generation with proper comma handling
// ---------------------------------------------------------------------------
fn emit_html_el(el: &Element, name_str: &str) -> TokenStream {
    emit_html_el_inner(el, name_str)
}

fn emit_html_el_inner(el: &Element, name_str: &str) -> TokenStream {
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

    let has_value = el.args.iter().any(|a| a.key == "value");
    let has_on_input = el.args.iter().any(|a| a.key == "on_input");
    let auto_bind_input = tag == "input" && has_value && !has_on_input;

    let mut attrs: Vec<TokenStream> = Vec::new();
    let mut children: Vec<TokenStream> = Vec::new();
    let mut children_attr_expr: Option<&syn::Expr> = None;

    for arg in &el.args {
        let key_str = arg.key.to_string();
        let key_ident = arg.key.clone();
        let value = &arg.value;
        match key_str.as_str() {
            // Special: "children" becomes a child expression, NOT an attribute
            "children" => {
                children_attr_expr = Some(value);
            }
            "on_click" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onclick: {#handler}, })
            }
            "on_mouse_down" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmousedown: {#handler}, })
            }
            "on_mouse_up" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseup: {#handler}, })
            }
            "on_mouse_enter" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseenter: {#handler}, })
            }
            "on_mouse_leave" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onmouseleave: {#handler}, })
            }
            "on_input" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { oninput: {#handler}, })
            }
            "on_change" => {
                let handler = wrap_dioxus_handler(value);
                attrs.push(quote! { onchange: {#handler}, })
            }
            "value" => {
                if tag == "input" {
                    // FIX: Use signal.get() directly, not string interpolation
                    attrs.push(quote! { value: {#value.get()}, });
                } else {
                    attrs.push(quote! { value: {#value}, });
                }
            }
            // Ignored attributes (handled elsewhere or not needed)
            "primary" | "ghost" | "destructive" | "active" | "trigger" | "rows" | "striped"
            | "items" | "estimated_height" | "copy_text" | "sortable" | "width" | "resizable"
            | "selectable" | "on_sort" | "bordered" | "size" | "navigate_to" | "cfg" | "label"
            | "render" | "key" | "index" | "text" | "query" | "color" | "direction" | "tooltip"
            | "icon_name" => {}
            // Generic attributes
            _ => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { #key_ident: #s, });
                } else {
                    attrs.push(quote! { #key_ident: {#value}, });
                }
            }
        }
    }

    if auto_bind_input {
        let value_expr = find_arg_expr(el, "value").unwrap();
        attrs.push(quote! {
            oninput: move |ev: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                #value_expr.set(ev.value());
            },
        });
    }

    // Add children from the `children` attribute (converted to iterator)
    if let Some(expr) = children_attr_expr {
        children.push(quote! { { #expr.into_iter() } });
    }

    // Add children from element's `children_expr`
    if let Some(children_expr) = &el.children_expr {
        children.push(quote! { { #children_expr.into_iter() } });
    }

    // Add nested child elements
    for child in &el.children {
        children.push(emit_render_inner(child));
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());

    // Build RSX element with correct comma handling
    if attrs.is_empty() && children.is_empty() {
        quote! { #tag_ident {} }
    } else if attrs.is_empty() {
        quote! { #tag_ident { #(#children)* } }
    } else if children.is_empty() {
        quote! { #tag_ident { #(#attrs)* } }
    } else {
        // Attributes already have trailing commas
        quote! { #tag_ident { #(#attrs)* #(#children)* } }
    }
}

// ---------------------------------------------------------------------------
// If node
// ---------------------------------------------------------------------------
fn emit_if(if_node: &IfNode) -> TokenStream {
    emit_if_inner(if_node)
}

fn emit_if_inner(if_node: &IfNode) -> TokenStream {
    let cond = &if_node.condition;
    let then_tokens = emit_nodes_inner(&if_node.then_branch);
    if let Some(else_branch) = &if_node.else_branch {
        if else_branch.len() == 1 {
            if let RenderNode::If(nested_if) = &else_branch[0] {
                let nested = emit_if_inner(nested_if);
                return quote! { if #cond { #then_tokens } else #nested };
            }
        }
        let else_tokens = emit_nodes_inner(else_branch);
        quote! { if #cond { #then_tokens } else { #else_tokens } }
    } else {
        quote! { if #cond { #then_tokens } }
    }
}

// ---------------------------------------------------------------------------
// For node
// ---------------------------------------------------------------------------
fn emit_for(for_node: &ForNode) -> TokenStream {
    emit_for_inner(for_node)
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

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------
fn emit_tabs(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let on_click_with_move = force_move_on_closure(on_click_expr);

        let tab_triggers: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;
                    let index_clone = index.clone();
                    Some(quote! {
                        shadcn_dioxus::tabs::TabsTrigger {
                            value: "{#index.to_string()}",
                            onclick: {
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#index_clone); }
                            },
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            shadcn_dioxus::tabs::Tabs {
                value: "{#active_expr.to_string()}",
                shadcn_dioxus::tabs::TabsList {
                    #(#tab_triggers)*
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let active_expr = find_arg_expr(el, "active").expect("tabs require 'active' argument");
        let on_click_expr =
            find_arg_expr(el, "on_click").expect("tabs require 'on_click' callback");

        let param_idents: Vec<proc_macro2::Ident> =
            if let syn::Expr::Closure(closure) = on_click_expr {
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

        let body_idents: Vec<proc_macro2::Ident> = collect_handler_idents(on_click_expr)
            .into_iter()
            .filter(|id| !param_names.contains(&id.to_string()))
            .collect();

        let on_click_with_move = force_move_on_closure(on_click_expr);

        let tab_elements: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "tab"
                {
                    let label = find_arg_expr(e, "label")?;
                    let index = find_arg_expr(e, "index")?;

                    let param_shadows: Vec<TokenStream> = param_idents
                        .iter()
                        .map(|id| quote! { let #id = #index; })
                        .collect();
                    let clone_shadows: Vec<TokenStream> = body_idents
                        .iter()
                        .map(|id| quote! { let #id = #id.clone(); })
                        .collect();
                    let call_args: Vec<TokenStream> =
                        param_idents.iter().map(|id| quote! { #id }).collect();

                    Some(quote! {
                        div {
                            class: if #index == #active_expr {
                                "px-4 py-2 cursor-pointer text-white"
                            } else {
                                "px-4 py-2 cursor-pointer text-gray-400"
                            },
                            onclick: {
                                #(#param_shadows)*
                                #(#clone_shadows)*
                                let __tab_on_click = #on_click_with_move;
                                move |_| { __tab_on_click(#(#call_args)*) }
                            },
                            {#label}
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            div { class: "flex", #(#tab_elements)* }
        }
    }
}
// ---------------------------------------------------------------------------
// Dropdown menu
// ---------------------------------------------------------------------------
fn emit_dropdown_menu(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { div { "dropdown: missing trigger" } },
        };

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;
                    let handler = wrap_dioxus_handler(on_click);
                    Some(quote! {
                        shadcn_dioxus::dropdown_menu::DropdownMenuItem {
                            onclick: {#handler},
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone()));

        quote! {
            shadcn_dioxus::dropdown_menu::DropdownMenu {
                shadcn_dioxus::dropdown_menu::DropdownMenuTrigger {
                    #trigger_inner
                }
                shadcn_dioxus::dropdown_menu::DropdownMenuContent {
                    #(#item_tokens)*
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let trigger_expr = match &el.trigger_expr {
            Some(e) => e,
            None => return quote! { div { "dropdown: missing trigger" } },
        };

        let item_tokens: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "item"
                {
                    let label = find_arg_expr(e, "label")?;
                    let on_click = find_arg_expr(e, "on_click")?;

                    let checked = find_arg_bool(e, "checked");

                    let handler = wrap_dioxus_handler(on_click);
                    let check_mark = if checked { "\u{2713} " } else { "" };
                    Some(quote! {
                        div {
                            class: "px-3 py-2 cursor-pointer text-white hover:bg-gray-600 flex items-center",
                            onclick: {
                                let __item_handler = #handler;
                                move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.stop_propagation();
                                    __open.set(false);
                                    __item_handler(ev);
                                }
                            },
                            #check_mark
                            #label
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        let trigger_inner = emit_render_inner(&RenderNode::Expr(trigger_expr.clone()));

        quote! {
            {
                let mut __open = dioxus::prelude::use_signal(|| false);
                dioxus::prelude::rsx! {
                    div {
                        class: "relative inline-block",
                        onclick: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                            ev.stop_propagation();
                            __open.toggle();
                        },
                        #trigger_inner,
                        if *__open.read() {
                            div {
                                class: "absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg",
                                onclick: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.stop_propagation();
                                },
                                onmousedown: move |ev: dioxus::prelude::Event<dioxus::prelude::MouseData>| {
                                    ev.prevent_default();
                                },
                                #(#item_tokens)*
                            }
                        }
                    }
                }
            }
        }
    }
}
// ---------------------------------------------------------------------------
// Data table
// ---------------------------------------------------------------------------
fn emit_data_table(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let rows = find_arg_expr(el, "rows").unwrap();

        let header_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let label = find_arg_expr(e, "label").unwrap();
                    Some(quote! { th { class: "px-3 py-2 text-gray-400 font-medium", #label } })
                } else {
                    None
                }
            })
            .collect();

        let row_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let render_closure = find_arg_expr(e, "render").unwrap();
                    Some(quote! { td { class: "px-3 py-2 text-white", { (#render_closure)(&__row) } } })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            table { class: "w-full text-sm",
                thead { tr { #(#header_cells)* } }
                tbody {
                    for __row in #rows {
                        tr { #(#row_cells)* }
                    }
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let rows = find_arg_expr(el, "rows").unwrap();

        let header_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let label = find_arg_expr(e, "label").unwrap();
                    Some(quote! { th { #label } })
                } else {
                    None
                }
            })
            .collect();

        let row_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let render_closure = find_arg_expr(e, "render").unwrap();
                    Some(quote! { td { { (#render_closure)(&__row) } } })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            table {
                thead { tr { #(#header_cells)* } }
                tbody {
                    for __row in #rows {
                        tr { #(#row_cells)* }
                    }
                }
            }
        }
    }
}
