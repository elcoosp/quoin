use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{
    collect_handler_idents, collect_handler_idents_excluding_params, force_move_on_closure,
};
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
        "dropdown_menu" => emit_dropdown_menu(el),
        "styled_text" => emit_styled_text(el),
        "badge" => emit_badge(el),
        "scroll_area" => emit_scroll_area(el),
        "virtual_list" => {
            let estimated_height = el
                .args
                .iter()
                .find(|a| a.key == "estimated_height")
                .and_then(|a| {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Float(f),
                        ..
                    }) = &a.value
                    {
                        f.base10_parse::<f32>().ok()
                    } else if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(i),
                        ..
                    }) = &a.value
                    {
                        i.base10_parse::<f32>().ok()
                    } else {
                        None
                    }
                });
            let children_tokens: Vec<TokenStream> =
                el.children.iter().map(emit_render_inner).collect();
            match estimated_height {
                Some(_h) => {
                    quote! { div { style: "overflow-y: auto; height: 100%", #(#children_tokens)* } }
                }
                None => {
                    quote! { div { style: "overflow-y: auto", #(#children_tokens)* } }
                }
            }
        }
        "clipboard_button" => emit_clipboard_button(el),
        "button" => emit_button(el),
        "icon" => emit_icon(el),
        "input" => emit_input(el),
        _ => emit_html_el(el, &name_str),
    }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

fn emit_badge(el: &Element) -> TokenStream {
    let color_expr = el.args.iter().find(|a| a.key == "color").map(|a| &a.value);
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_render_inner(child));
    }
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

// ---------------------------------------------------------------------------
// Scroll area
// ---------------------------------------------------------------------------

fn emit_scroll_area(el: &Element) -> TokenStream {
    let direction = el
        .args
        .iter()
        .find(|a| a.key == "direction")
        .and_then(|a| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &a.value
            {
                Some(s.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "vertical".to_string());

    let overflow_class = match direction.as_str() {
        "horizontal" => "overflow-x-auto",
        "both" => "overflow-auto",
        _ => "overflow-y-auto",
    };

    let mut items = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => items.push(quote! { class: format!("{} {}", #value, #overflow_class) }),
            "direction" => {}
            _ => {}
        }
    }
    if items.is_empty() {
        items.push(quote! { class: #overflow_class });
    }
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_render_inner(child));
    }
    quote! { div { #(#items),* #(#children)* } }
}

// ---------------------------------------------------------------------------
// Button — with optional tooltip wrapping
// ---------------------------------------------------------------------------

fn emit_button(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        emit_button_shadcn(el)
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_button_plain(el)
    }
}

fn emit_button_plain(el: &Element) -> TokenStream {
    let tooltip_text = el.args.iter().find(|a| a.key == "tooltip").and_then(|a| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &a.value
        {
            Some(s.value())
        } else {
            None
        }
    });

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

#[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
fn emit_button_shadcn(el: &Element) -> TokenStream {
    let tooltip_text = el.args.iter().find(|a| a.key == "tooltip").and_then(|a| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &a.value
        {
            Some(s.value())
        } else {
            None
        }
    });

    let class_expr = el.args.iter().find(|a| a.key == "class").map(|a| &a.value);

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

    let mut on_click_attr: Option<TokenStream> = None;
    if let Some(handler_expr) = el
        .args
        .iter()
        .find(|a| a.key == "on_click")
        .map(|a| &a.value)
    {
        let handler = wrap_dioxus_handler(handler_expr);
        on_click_attr = Some(quote! { onclick: #handler })
    }

    let mut children = Vec::new();
    for child in &el.children {
        children.push(emit_render_inner(child));
    }

    let inner_button = if children.is_empty() {
        quote! {
            button { class: #full_class, #on_click_attr }
        }
    } else {
        quote! {
            button { class: #full_class, #on_click_attr
                #(#children)*
            }
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

// ---------------------------------------------------------------------------
// Input — plain HTML vs shadcn
// ---------------------------------------------------------------------------

fn emit_input(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        emit_input_shadcn(el)
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_input_plain(el)
    }
}

fn emit_input_plain(el: &Element) -> TokenStream {
    emit_html_el_inner(el, "input")
}

#[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
fn emit_input_shadcn(el: &Element) -> TokenStream {
    let placeholder = el
        .args
        .iter()
        .find(|a| a.key == "placeholder")
        .and_then(|a| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &a.value
            {
                Some(s.value())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let class_expr = el.args.iter().find(|a| a.key == "class").map(|a| &a.value);
    let value_expr = el.args.iter().find(|a| a.key == "value").map(|a| &a.value);
    let on_input_expr = el
        .args
        .iter()
        .find(|a| a.key == "on_input")
        .map(|a| &a.value);
    let disabled = find_arg_bool(el, "disabled");

    let base_class = "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

    let full_class = match class_expr {
        Some(cls) => quote! { format!("{} {}", #base_class, #cls) },
        None => quote! { #base_class },
    };

    let placeholder_attr = if placeholder.is_empty() {
        quote! {}
    } else {
        quote! { placeholder: #placeholder }
    };

    let disabled_attr = if disabled {
        quote! { disabled: true }
    } else {
        quote! {}
    };

    let value_attr = if let Some(val) = value_expr {
        quote! { value: "{#val.get()}" }
    } else {
        quote! {}
    };

    let oninput_attr = if let Some(handler) = on_input_expr {
        let wrapped = wrap_dioxus_handler(handler);
        quote! { oninput: #wrapped }
    } else if value_expr.is_some() {
        let sig = value_expr.unwrap();
        quote! { oninput: move |ev: dioxus::prelude::Event<web_sys::InputEvent>| { #sig.set(ev.value()); } }
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

// ---------------------------------------------------------------------------
// Icon — inline SVG from icon_codegen
// ---------------------------------------------------------------------------

fn emit_icon(el: &Element) -> TokenStream {
    let name = el.args.iter().find(|a| a.key == "icon_name").and_then(|a| {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &a.value
        {
            Some(s.value())
        } else {
            None
        }
    });

    let size_class = el.args.iter().find(|a| a.key == "class").map(|a| &a.value);

    let class_str = match size_class {
        Some(c) => quote! { format!("{} w-4 h-4 inline-block", #c) },
        None => quote! { "w-4 h-4 inline-block" },
    };

    let children: Vec<TokenStream> = el.children.iter().map(emit_render_inner).collect();

    match name {
        Some(n) => {
            if let Some(svg) = crate::transpile::icon_codegen::icon_to_svg(&n) {
                quote! { span { class: #class_str, #svg } }
            } else {
                quote! { span { class: #class_str, "❓" } }
            }
        }
        None => {
            if children.is_empty() {
                quote! { span { class: #class_str, "❓" } }
            } else {
                quote! { span { class: #class_str, #(#children)* } }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// StyledText — text with optional search highlighting
// ---------------------------------------------------------------------------

fn emit_styled_text(el: &Element) -> TokenStream {
    let text_expr = el.args.iter().find(|a| a.key == "text").map(|a| &a.value);
    let query_expr = el.args.iter().find(|a| a.key == "query").map(|a| &a.value);

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
    let copy_text = match el.args.iter().find(|a| a.key == "copy_text").map(|a| &a.value) {
        Some(ct) => ct,
        None => return emit_html_el(el, "button"),
    };

    let mut attrs = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class: #value }),
            "disabled" => attrs.push(quote! { disabled: #value }),
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
            #(#attrs),*
            onclick: move |_| {
                quoin::clipboard_write_text(&(#copy_text).to_string());
            },
            #(#children)*
        }
    }
}

// ---------------------------------------------------------------------------
// HTML element emitter
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
                    items.push(quote! { value: "{#value.get()}" });
                } else {
                    items.push(quote! { value: {#value} });
                }
            }
            "primary" | "ghost" | "destructive" | "active" | "children" | "trigger" | "rows"
            | "striped" | "items" | "estimated_height" | "copy_text" | "sortable" | "width"
            | "resizable" | "selectable" | "on_sort" | "bordered" | "size" | "navigate_to"
            | "cfg" | "label" | "render" | "key" | "index" | "text" | "query" | "color"
            | "direction" | "tooltip" | "icon_name" => {}
            _ => {
                let key = proc_macro2::Ident::new(&key_str, proc_macro2::Span::call_site());
                items.push(quote! { #key: {#value} });
            }
        }
    }

    if auto_bind_input {
        let value_expr = el
            .args
            .iter()
            .find(|a| a.key == "value")
            .map(|a| &a.value)
            .unwrap();
        items.push(quote! {
            oninput: move |ev: dioxus::prelude::Event<web_sys::InputEvent>| {
                #value_expr.set(ev.value());
            }
        });
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

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

fn emit_tabs(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        return emit_tabs_shadcn(el);
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_tabs_plain(el)
    }
}

fn emit_tabs_plain(el: &Element) -> TokenStream {
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

    let body_idents: Vec<proc_macro2::Ident> = collect_handler_idents(on_click_expr)
        .into_iter()
        .filter(|id| !param_names.contains(&id.to_string()))
        .collect();

    let on_click_with_move = force_move_on_closure(on_click_expr);

    let per_tab: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "tab"
            {
                let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;

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

                return Some(quote! {
                    div {
                        class: if #index == __active { "px-4 py-2 cursor-pointer text-white" } else { "px-4 py-2 cursor-pointer text-gray-400" },
                        onclick: {
                            #(#param_shadows)*
                            #(#clone_shadows)*
                            let __tab_on_click = #on_click_with_move;
                            move |_| { __tab_on_click(#(#call_args)*) }
                        },
                        #label
                    }
                });
            }
            None
        })
        .collect();

    quote! {
        {
            let __active = #active_expr;
            dioxus::prelude::rsx! {
                div { class: "flex",
                    #(#per_tab)*
                }
            }
        }
    }
}

#[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
fn emit_tabs_shadcn(el: &Element) -> TokenStream {
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

    let on_click_with_move = force_move_on_closure(on_click_expr);

    let tab_triggers: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "tab"
            {
                let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;
                let index_clone = index.clone();
                return Some(quote! {
                    shadcn_dioxus::tabs::TabsTrigger {
                        value: "{#index.to_string()}",
                        onclick: {
                            let __tab_on_click = #on_click_with_move;
                            move |_| { __tab_on_click(#index_clone); }
                        },
                        #label
                    }
                });
            }
            None
        })
        .collect();

    quote! {
        {
            let __active = #active_expr;
            dioxus::prelude::rsx! {
                shadcn_dioxus::tabs::Tabs {
                    value: "{__active}",
                    shadcn_dioxus::tabs::TabsList {
                        #(#tab_triggers)*
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dropdown menu
// ---------------------------------------------------------------------------

fn emit_dropdown_menu(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        return emit_dropdown_menu_shadcn(el);
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_dropdown_menu_plain(el)
    }
}

fn emit_dropdown_menu_plain(el: &Element) -> TokenStream {
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
                let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let on_click = e
                    .args
                    .iter()
                    .find(|a| a.key == "on_click")
                    .map(|a| &a.value)?;

                let handler = wrap_dioxus_handler(on_click);
                Some(quote! {
                    div {
                        class: "px-3 py-2 cursor-pointer text-white hover:bg-gray-600",
                        onclick: {
                            let __item_handler = #handler;
                            move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                                ev.stop_propagation();
                                __open.set(false);
                                __item_handler(ev);
                            }
                        },
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
                    onclick: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                        ev.stop_propagation();
                        __open.toggle();
                    },
                    #trigger_inner,
                    if *__open.read() {
                        div {
                            class: "absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg",
                            onclick: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
                                ev.stop_propagation();
                            },
                            onmousedown: move |ev: dioxus::prelude::Event<web_sys::MouseEvent>| {
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

#[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
fn emit_dropdown_menu_shadcn(el: &Element) -> TokenStream {
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
                let label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let on_click = e.args.iter().find(|a| a.key == "on_click").map(|a| &a.value)?;
                let handler = wrap_dioxus_handler(on_click);
                Some(quote! {
                    shadcn_dioxus::dropdown_menu::DropdownMenuItem {
                        onclick: #handler,
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
        dioxus::prelude::rsx! {
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
}

fn emit_data_table(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        return emit_data_table_shadcn(el);
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        emit_data_table_plain(el)
    }
}

fn emit_data_table_plain(el: &Element) -> TokenStream {
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
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let label = e
                    .args
                    .iter()
                    .find(|a| a.key == "label")
                    .map(|a| &a.value)
                    .unwrap();
                return Some(quote!(th { #label }));
            }
            None
        })
        .collect();

    let row_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let render_closure = e
                    .args
                    .iter()
                    .find(|a| a.key == "render")
                    .map(|a| &a.value)
                    .unwrap();
                return Some(quote!(td { { (#render_closure)(&__row) } }));
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

#[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
fn emit_data_table_shadcn(el: &Element) -> TokenStream {
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
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let label = e
                    .args
                    .iter()
                    .find(|a| a.key == "label")
                    .map(|a| &a.value)
                    .unwrap();
                return Some(quote!(th { class: "px-3 py-2 text-gray-400 font-medium", #label }));
            }
            None
        })
        .collect();

    let row_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let render_closure = e
                    .args
                    .iter()
                    .find(|a| a.key == "render")
                    .map(|a| &a.value)
                    .unwrap();
                return Some(quote!(td { class: "px-3 py-2 text-white", { (#render_closure)(&__row) } }));
            }
            None
        })
        .collect();

    quote! {
        dioxus::prelude::rsx! {
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
}

#[allow(dead_code)]
fn find_arg_bool(el: &Element, key: &str) -> bool {
    el.args
        .iter()
        .find(|a| a.key == key)
        .map(|a| {
            if let syn::Expr::Lit(expr_lit) = &a.value
                && let syn::Lit::Bool(b) = &expr_lit.lit
            {
                return b.value;
            }
            false
        })
        .unwrap_or(false)
}
