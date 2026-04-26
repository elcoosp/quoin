use crate::render_ast::{Element, ForNode, IfNode, RenderNode};
use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;
use std::sync::atomic::{AtomicUsize, Ordering};

static EXTRACT_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_extract_id() -> usize {
    EXTRACT_COUNTER.fetch_add(1, Ordering::Relaxed)
}


/// Returns the element identifier to use in view!.
/// - shadcn ON:  imports the shadcn component as a local alias, returns the alias ident
/// - shadcn OFF: returns the plain HTML tag ident
#[cfg(feature = "leptos-shadcn")]
fn import_shadcn_or_html_tag(
    bindings: &mut Vec<TokenStream>,
    shadcn_comp: &str,
    _html_tag: &str,
) -> proc_macro2::Ident {
    let alias = quote::format_ident!("{}_{}", shadcn_comp, next_extract_id());
    let comp_ident = quote::format_ident!("{}", shadcn_comp);
    bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
    alias
}

#[cfg(not(feature = "leptos-shadcn"))]
fn import_shadcn_or_html_tag(
    _bindings: &mut Vec<TokenStream>,
    _shadcn_comp: &str,
    html_tag: &str,
) -> proc_macro2::Ident {
    quote::format_ident!("{}", html_tag)
}

pub fn emit_render(node: &RenderNode) -> TokenStream {
    let mut bindings = Vec::new();
    let inner = emit_node(node, &mut bindings, false);

    let tokens = if bindings.is_empty() {
        quote! { { use leptos::prelude::*; leptos::view! { #inner } } }
    } else {
        quote! { { use leptos::prelude::*; #(#bindings)* leptos::view! { #inner } } }
    };

    wrap_with_cfg(node.attrs(), tokens)
}

fn emit_node(node: &RenderNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el, bindings, inside_for),
        RenderNode::Text(t) => quote! { #t },
        RenderNode::Expr(e) => {
            if inside_for {
                quote! { {#e} }
            } else {
                let expr_id = next_extract_id();
                let expr_name = quote::format_ident!("__quoin_expr_{}", expr_id);
                bindings.push(quote! { let #expr_name = (#e).clone(); });
                quote! { {#expr_name.clone()} }
            }
        }
        RenderNode::If(if_node) => emit_if(if_node, bindings, inside_for),
        RenderNode::For(for_node) => emit_for(for_node, bindings),
        RenderNode::Root(nodes) => {
            let tokens: Vec<TokenStream> = nodes
                .iter()
                .map(|n| emit_node(n, bindings, inside_for))
                .collect();
            if tokens.len() == 1 {
                tokens[0].clone()
            } else {
                quote! { <> #(#tokens)* </> }
            }
        }
    }
}

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
// If nodes
// ---------------------------------------------------------------------------

fn emit_if(if_node: &IfNode, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let inner = emit_if_reactive(if_node, bindings, inside_for);
    wrap_with_cfg(&if_node.attrs, inner)
}

fn emit_if_reactive(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let mut cond_bindings = Vec::new();
    let body = build_if_expr_extracting_conds(if_node, bindings, inside_for, &mut cond_bindings);

    quote! {
        {
            #(#cond_bindings)*
            move || { use leptos::prelude::*; #body }
        }
    }
}

fn build_if_expr_extracting_conds(
    if_node: &IfNode,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
    cond_bindings: &mut Vec<TokenStream>,
) -> TokenStream {
    let cond_id = next_extract_id();
    let cond_name = quote::format_ident!("__quoin_if_cond_{}", cond_id);
    let cond_expr = &if_node.condition;

    cond_bindings.push(quote! { let #cond_name = #cond_expr; });

    let then_tokens: Vec<TokenStream> = if_node
        .then_branch
        .iter()
        .map(|n| emit_node(n, bindings, inside_for))
        .collect();
    let then_view = quote! { #(#then_tokens)* };

    match &if_node.else_branch {
        Some(else_branch) => {
            if else_branch.len() == 1 {
                if let RenderNode::If(nested_if) = &else_branch[0] {
                    let nested_body = build_if_expr_extracting_conds(
                        nested_if,
                        bindings,
                        inside_for,
                        cond_bindings,
                    );
                    return quote! {
                        if #cond_name {
                            { leptos::view! { #then_view } }.into_any()
                        } else {
                            #nested_body
                        }
                    };
                }
            }
            let else_tokens: Vec<TokenStream> = else_branch
                .iter()
                .map(|n| emit_node(n, bindings, inside_for))
                .collect();
            let else_view = quote! { #(#else_tokens)* };
            quote! {
                if #cond_name {
                    { leptos::view! { #then_view } }.into_any()
                } else {
                    { leptos::view! { #else_view } }.into_any()
                }
            }
        }
        None => {
            quote! {
                (#cond_name).then(|| { leptos::view! { #then_view } }.into_any())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// For nodes
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
    let iter_name = quote::format_ident!("__quoin_for_iter_{}", iter_id);
    bindings.push(quote! { let #iter_name = (#iterable).clone(); });

    quote! {
        {
            #iter_name.clone().into_iter().map(|#pat| {
                leptos::view! { #body_view }
            }).collect::<Vec<_>>()
        }
    }
}


// ---------------------------------------------------------------------------
// Separator (Tier 1 — no variant, just tag swap)
// ---------------------------------------------------------------------------

fn resolve_separator_element(
    bindings: &mut Vec<TokenStream>,
    el: &Element,
) -> proc_macro2::Ident {
    let orientation = find_arg_string(el, "orientation").unwrap_or_else(|| "horizontal".to_string());
    let html_tag = if orientation == "horizontal" { "hr" } else { "div" };
    import_shadcn_or_html_tag(bindings, "Separator", html_tag)
}

fn emit_separator(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = resolve_separator_element(bindings, el);
    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=#value }),
            "orientation" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        quote! { <#tag /> }
    } else {
        quote! { <#tag #(#attrs)* /> }
    }
}

// ---------------------------------------------------------------------------
// Skeleton / SkeletonText / SkeletonAvatar (Tier 1 — no variant, just tag swap)
// ---------------------------------------------------------------------------

fn emit_skeleton(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse rounded-md bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}

fn emit_skeleton_text(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse h-4 w-full rounded-md bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}

fn emit_skeleton_avatar(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let tag = import_shadcn_or_html_tag(bindings, "Skeleton", "div");
    let base = "animate-pulse h-10 w-10 rounded-full bg-muted";
    let user_class = find_arg_string(el, "class").unwrap_or_default();
    let full_class = if user_class.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, user_class)
    };
    quote! { <#tag class=#full_class /> }
}

// ---------------------------------------------------------------------------
// Progress (Tier 2 — variant: determinate value vs indeterminate)
// ---------------------------------------------------------------------------

fn emit_progress(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let value_expr = el.args.iter().find(|a| a.key == "value").map(|a| &a.value);
    let max_expr = el.args.iter().find(|a| a.key == "max").map(|a| &a.value);
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = import_shadcn_or_html_tag(bindings, "Progress", "div");
        let value_prop = match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) },
                    None => quote! { (#val as f64) / 100.0 },
                };
                quote! { value={leptos::prelude::Signal::derive(move || #max)} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() { quote! {} } else { quote! { class={#user_class} } };
        quote! { <#tag #value_prop #class_prop /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let outer_cls = if user_class.is_empty() {
            "relative h-4 w-full overflow-hidden rounded-full bg-secondary".to_string()
        } else {
            format!("relative h-4 w-full overflow-hidden rounded-full bg-secondary {}", user_class)
        };
        let bar_cls = "h-full rounded-full bg-primary transition-all duration-300";

        match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) * 100.0 },
                    None => quote! { (#val as f64) },
                };
                let val_id = next_extract_id();
                let val_name = quote::format_ident!("__quoin_prog_val_{}", val_id);
                bindings.push(quote! { let #val_name = #max; });
                quote! {
                    <div class=#outer_cls>
                        <div class=#bar_cls style={leptos::prelude::Signal::derive(move || format!("width: {}%", #val_name))} />
                    </div>
                }
            }
            None => {
                // Indeterminate: animated sliding bar
                let indeterminate_cls = "h-full w-1/3 rounded-full bg-primary animate-indeterminate";
                quote! {
                    <div class=#outer_cls>
                        <div class=#indeterminate_cls />
                    </div>
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Checkbox (Tier 2 — type=checkbox vs shadcn Checkbox)
// ---------------------------------------------------------------------------

fn emit_checkbox(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let checked_expr = el.args.iter().find(|a| a.key == "checked").map(|a| &a.value);
    let on_change_expr = el.args.iter().find(|a| a.key == "on_checked_change").or_else(|| {
        el.args.iter().find(|a| a.key == "on_change")
    }).map(|a| &a.value);
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = import_shadcn_or_html_tag(bindings, "Checkbox", "input");
        let checked_prop = match checked_expr {
            Some(val) => quote! { checked={#val} },
            None => quote! {},
        };
        let on_change_prop = match on_change_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_checked_change={#wrapped} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() { quote! {} } else { quote! { class={#user_class} } };
        quote! { <#tag #checked_prop #on_change_prop #class_prop disabled={#disabled} /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let base = "h-4 w-4 rounded border border-input ring-offset-background accent-primary-500 cursor-pointer";
        let full_class = if user_class.is_empty() { base.to_string() } else { format!("{} {}", base, user_class) };

        let checked_prop = match checked_expr {
            Some(val) => {
                quote! { prop:checked={leptos::prelude::Signal::derive(move || #val)} }
            }
            None => quote! {},
        };

        let on_input_prop = match on_change_expr {
            Some(handler) => {
                let handler = wrap_event_handler(handler);
                let bind_id = next_extract_id();
                let bind_name = quote::format_ident!("__quoin_cb_bind_{}", bind_id);
                bindings.push(quote! {
                    let #bind_name = {
                        let __handler = #handler;
                        move |ev: leptos::ev::Event| {
                            let checked = leptos::prelude::event_target_checked(&ev);
                            __handler(checked);
                        }
                    };
                });
                quote! { on:input=#bind_name }
            }
            None => quote! {},
        };

        let disabled_prop = if disabled { quote! { disabled=true } } else { quote! {} };
        let type_prop = quote! { r#type="checkbox"# };

        // Build attrs list
        let mut attrs: Vec<TokenStream> = vec![
            quote! { class=#full_class },
            type_prop,
            checked_prop,
            on_input_prop,
            disabled_prop,
        ];

        let tag_ident = proc_macro2::Ident::new("input", proc_macro2::Span::call_site());
        quote! { <#tag_ident #(#attrs)* /> }
    }
}

// ---------------------------------------------------------------------------
// Switch (Tier 2 — toggle-switch styled checkbox)
// ---------------------------------------------------------------------------

fn emit_switch(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let checked_expr = el.args.iter().find(|a| a.key == "checked").map(|a| &a.value);
    let on_change_expr = el.args.iter().find(|a| a.key == "on_checked_change").or_else(|| {
        el.args.iter().find(|a| a.key == "on_change")
    }).map(|a| &a.value);
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = import_shadcn_or_html_tag(bindings, "Switch", "button");
        let checked_prop = match checked_expr {
            Some(val) => quote! { checked={#val} },
            None => quote! {},
        };
        let on_change_prop = match on_change_expr {
            Some(handler) => {
                let wrapped = wrap_event_handler(handler);
                quote! { on_checked_change={#wrapped} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() { quote! {} } else { quote! { class={#user_class} } };
        quote! { <#tag #checked_prop #on_change_prop #class_prop disabled={#disabled} /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let track_cls = "relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors";
        let track_on_cls = "bg-primary";
        let track_off_cls = "bg-input";
        let thumb_cls = "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform";
        let thumb_on_cls = "translate-x-5";
        let thumb_off_cls = "translate-x-0";
        let disabled_cls = if disabled { " opacity-50 pointer-events-none" } else { "" };

        let checked_prop = match checked_expr {
            Some(val) => {
                quote! { prop:checked={leptos::prelude::Signal::derive(move || #val)} }
            }
            None => quote! {},
        };

        let on_input_prop = match on_change_expr {
            Some(handler) => {
                let handler = wrap_event_handler(handler);
                let bind_id = next_extract_id();
                let bind_name = quote::format_ident!("__quoin_sw_bind_{}", bind_id);
                bindings.push(quote! {
                    let #bind_name = {
                        let __handler = #handler;
                        move |ev: leptos::ev::Event| {
                            let checked = leptos::prelude::event_target_checked(&ev);
                            __handler(checked);
                        }
                    };
                });
                quote! { on:input=#bind_name }
            }
            None => quote! {},
        };

        let role_prop = quote! { role="switch" };
        let type_prop = quote! { r#type="checkbox"# };

        let track_class_id = next_extract_id();
        let track_class_name = quote::format_ident!("__quoin_sw_track_{}", track_class_id);
        let thumb_class_id = next_extract_id();
        let thumb_class_name = quote::format_ident!("__quoin_sw_thumb_{}", thumb_class_id);
        bindings.push(quote! {
            let #track_class_name = leptos::prelude::Signal::derive(move || {
                if #checked_expr.map(|v| v.clone()).unwrap_or(false) {
                    concat!(#track_on_cls, #disabled_cls)
                } else {
                    concat!(#track_off_cls, #disabled_cls)
                }
            });
            let #thumb_class_name = leptos::prelude::Signal::derive(move || {
                if #checked_expr.map(|v| v.clone()).unwrap_or(false) {
                    concat!(#thumb_cls, " ", #thumb_on_cls)
                } else {
                    concat!(#thumb_cls, " ", #thumb_off_cls)
                }
            });
        });

        let full_cls = if user_class.is_empty() { track_cls.to_string() } else { format!("{} {}", track_cls, user_class) };

        quote! {
            <button
                type="button"
                role="switch"
                class=#full_cls
                disabled={#disabled}
                on:click=move |_| {}
            >
                <input #type_prop #checked_prop #on_input_prop class="sr-only peer" />
                <div class={#track_class_name} />
                <div class={#thumb_class_name} />
            </button>
        }
    }
}

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
        "separator" => emit_separator(el, bindings, inside_for),
        "skeleton" => emit_skeleton(el, bindings, inside_for),
        "skeleton_text" => emit_skeleton_text(el, bindings, inside_for),
        "skeleton_avatar" => emit_skeleton_avatar(el, bindings, inside_for),
        "progress" => emit_progress(el, bindings, inside_for),
        "checkbox" => emit_checkbox(el, bindings, inside_for),
        "switch" => emit_switch(el, bindings, inside_for),
        "tabs" => emit_tabs(el, bindings, inside_for),
        "data_table" => emit_data_table(el, bindings, inside_for),
        "dropdown_menu" => emit_dropdown_menu(el, bindings, inside_for),
        "styled_text" => emit_styled_text(el, bindings, inside_for),
        "badge" => emit_badge(el, bindings, inside_for),
        "scroll_area" => emit_scroll_area(el, bindings, inside_for),
        "virtual_list" => {
            let children_tokens: Vec<TokenStream> = el
                .children
                .iter()
                .map(|c| emit_node(c, bindings, inside_for))
                .collect();
            quote! { <div style="overflow-y: auto"> #(#children_tokens)* </div> }
        }
        "clipboard_button" => emit_clipboard_button(el, bindings, inside_for),
        "button" => emit_button(el, bindings, inside_for),
        "input" => emit_input(el, bindings, inside_for),
        "icon" => emit_icon(el, bindings, inside_for),
        _ => emit_html_tag(
            el,
            match name_str.as_str() {
                "div" => "div",
                "h1" => "h1",
                "h2" => "h2",
                "h3" => "h3",
                "p" | "text" => "p",
                "span" => "span",
                "a" => "a",
                "ul" => "ul",
                "li" => "li",
                "label" => "label",
                "textarea" => "textarea",
                "select" => "select",
                "form" => "form",
                "img" => "img",
                _ => "div",
            },
            bindings,
            inside_for,
        ),
    }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------
fn emit_badge(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_badge_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_badge_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_badge_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let color_expr = el.args.iter().find(|a| a.key == "color").map(|a| &a.value);
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }
    match color_expr {
        Some(color) => {
            let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
            match bg_class {
                Some(cls) => quote! {
                    <span class={concat!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white ", #cls)}>
                        #(#children)*
                    </span>
                },
                None => quote! {
                    <span
                        class="inline-flex items-center px-1.5 rounded text-xs font-medium text-white"
                        style=format!("background-color: {}", #color)
                    >
                        #(#children)*
                    </span>
                },
            }
        }
        None => quote! {
            <span class="inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white">
                #(#children)*
            </span>
        },
    }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_badge_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let color_expr = el.args.iter().find(|a| a.key == "color").map(|a| &a.value);
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }

    let badge_alias = quote::format_ident!("Badge_{}", next_extract_id());
    bindings.push(quote! {
        let #badge_alias = leptos_shadcn_ui::Badge;
    });

    let class_prop = if let Some(color) = color_expr {
        let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
        match bg_class {
            Some(cls) => {
                quote! { class={format!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}", #cls)} }
            }
            None => {
                quote! { class="inline-flex items-center px-1.5 rounded text-xs font-medium text-white" }
            }
        }
    } else {
        quote! { class="inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white" }
    };

    if children.is_empty() {
        quote! { <#badge_alias #class_prop /> }
    } else {
        quote! { <#badge_alias #class_prop> #(#children)* </#badge_alias> }
    }
}

// ---------------------------------------------------------------------------
// Scroll area
// ---------------------------------------------------------------------------
fn emit_scroll_area(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_scroll_area_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_scroll_area_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_scroll_area_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
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

    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => attrs.push(quote! { class=format!("{} {}", #value, #overflow_class) }),
            "direction" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        attrs.push(quote! { class=#overflow_class });
    }

    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }
    quote! { <div #(#attrs)*> #(#children)* </div> }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_scroll_area_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let class_expr = el.args.iter().find(|a| a.key == "class").map(|a| &a.value);
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }

    let class_prop = if let Some(cls) = class_expr {
        quote! { class={#cls} }
    } else {
        quote! {}
    };

    let sa_alias = quote::format_ident!("ScrollArea_{}", next_extract_id());
    bindings.push(quote! {
        let #sa_alias = leptos_shadcn_ui::ScrollArea;
    });

    quote! { <#sa_alias #class_prop> #(#children)* </#sa_alias> }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------
fn emit_button(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_button_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_button_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_button_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
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

    let inner_button = emit_html_tag_inner(el, "button", bindings, inside_for);

    match tooltip_text {
        Some(text) => quote! {
            <div class="relative inline-block group">
                #inner_button
                <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-2 py-1 text-xs rounded bg-gray-800 text-white whitespace-nowrap shadow-lg z-50 hidden group-hover:block">
                    {#text}
                </div>
            </div>
        },
        None => inner_button,
    }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_button_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
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

    let primary = find_arg_bool(el, "primary");
    let destructive = find_arg_bool(el, "destructive");
    let ghost = find_arg_bool(el, "ghost");
    let disabled = find_arg_bool(el, "disabled");

    let btn_alias = quote::format_ident!("Button_{}", next_extract_id());
    bindings.push(quote! {
        let #btn_alias = leptos_shadcn_ui::Button;
    });

    let variant = if destructive {
        quote! { { leptos_shadcn_ui::ButtonVariant::Destructive } }
    } else if ghost {
        quote! { { leptos_shadcn_ui::ButtonVariant::Ghost } }
    } else if primary {
        quote! { { leptos_shadcn_ui::ButtonVariant::Default } }
    } else {
        quote! { { leptos_shadcn_ui::ButtonVariant::Outline } }
    };

    let on_click_prop: Option<TokenStream> = if let Some(handler_expr) = el
        .args
        .iter()
        .find(|a| a.key == "on_click")
        .map(|a| &a.value)
    {
        let handler = wrap_event_handler(handler_expr);
        Some(quote! { on_click={#handler} })
    } else {
        None
    };

    let class_prop: TokenStream =
        if let Some(cls) = el.args.iter().find(|a| a.key == "class").map(|a| &a.value) {
            quote! { class={#cls} }
        } else {
            quote! {}
        };

    let mut children = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }

    let button = if children.is_empty() {
        let props = match on_click_prop {
            Some(oc) => quote! { variant=#variant #class_prop #oc disabled={#disabled} },
            None => quote! { variant=#variant #class_prop disabled={#disabled} },
        };
        quote! { <#btn_alias #props /> }
    } else {
        let props = match on_click_prop {
            Some(oc) => quote! { variant=#variant #class_prop #oc disabled={#disabled} },
            None => quote! { variant=#variant #class_prop disabled={#disabled} },
        };
        quote! { <#btn_alias #props> #(#children)* </#btn_alias> }
    };

    let wrapped = if let Some(text) = tooltip_text {
        let tp_alias = quote::format_ident!("TooltipProvider_{}", next_extract_id());
        let tt_alias = quote::format_ident!("Tooltip_{}", next_extract_id());
        let ttr_alias = quote::format_ident!("TooltipTrigger_{}", next_extract_id());
        let ttc_alias = quote::format_ident!("TooltipContent_{}", next_extract_id());

        bindings.push(quote! {
            let #tp_alias = leptos_shadcn_ui::TooltipProvider;
            let #tt_alias = leptos_shadcn_ui::Tooltip;
            let #ttr_alias = leptos_shadcn_ui::TooltipTrigger;
            let #ttc_alias = leptos_shadcn_ui::TooltipContent;
        });

        quote! {
            <#tp_alias>
                <#tt_alias>
                    <#ttr_alias>
                        #button
                    </#ttr_alias>
                    <#ttc_alias>
                        {#text}
                    </#ttc_alias>
                </#tt_alias>
            </#tp_alias>
        }
    } else {
        button
    };

    wrapped
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------
fn emit_input(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_input_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_input_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_input_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    emit_html_tag_inner(el, "input", bindings, inside_for)
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_input_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
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
    let on_change_expr = el
        .args
        .iter()
        .find(|a| a.key == "on_change")
        .map(|a| &a.value);
    let on_input_expr = el
        .args
        .iter()
        .find(|a| a.key == "on_input")
        .map(|a| &a.value);
    let disabled = find_arg_bool(el, "disabled");

    let has_explicit_handler = on_change_expr.is_some() || on_input_expr.is_some();
    let needs_auto_bind = value_expr.is_some() && !has_explicit_handler;

    let value_prop: TokenStream = if let Some(val) = value_expr {
        quote! {
            value={
                let __val = (#val).clone();
                leptos::prelude::Signal::derive(move || __val.get())
            }
        }
    } else {
        quote! {}
    };

    let on_change_prop: TokenStream = if let Some(handler) = on_change_expr {
        let wrapped = wrap_event_handler(handler);
        quote! { on_change={#wrapped} }
    } else if let Some(handler) = on_input_expr {
        let wrapped = wrap_event_handler(handler);
        quote! { on_change={#wrapped} }
    } else if needs_auto_bind {
        let val = value_expr.unwrap();
        let bind_id = next_extract_id();
        let bind_name = quote::format_ident!("__quoin_input_bind_{}", bind_id);
        bindings.push(quote! {
            let #bind_name = {
                let __signal = (#val).clone();
                move |val: String| {
                    __signal.set(val);
                }
            };
        });
        quote! { on_change=#bind_name }
    } else {
        quote! {}
    };

    let placeholder_prop: TokenStream = if placeholder.is_empty() {
        quote! {}
    } else {
        quote! { placeholder={#placeholder} }
    };

    let class_prop: TokenStream = if let Some(cls) = class_expr {
        quote! { class={#cls} }
    } else {
        quote! {}
    };

    let disabled_prop: TokenStream = if disabled {
        quote! { disabled=true }
    } else {
        quote! {}
    };

    let input_alias = quote::format_ident!("Input_{}", next_extract_id());
    bindings.push(quote! {
        let #input_alias = leptos_shadcn_ui::Input;
    });

    quote! { <#input_alias #value_prop #on_change_prop #placeholder_prop #class_prop #disabled_prop /> }
}

// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------
fn emit_icon(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
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
    let children: Vec<TokenStream> = el
        .children
        .iter()
        .map(|c| emit_node(c, bindings, inside_for))
        .collect();

    match name {
        Some(n) => {
            if let Some(svg) = crate::transpile::icon_codegen::icon_svg_html(&n) {
                quote! {
                    <span class=#class_str>
                        #svg
                    </span>
                }
            } else {
                quote! {
                    <span class=#class_str>"❓"</span>
                }
            }
        }
        None => {
            if children.is_empty() {
                quote! {
                    <span class=#class_str>"❓"</span>
                }
            } else {
                quote! {
                    <span class=#class_str>
                        #(#children)*
                    </span>
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// StyledText
// ---------------------------------------------------------------------------
fn emit_styled_text(
    el: &Element,
    _bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let text_expr = el.args.iter().find(|a| a.key == "text").map(|a| &a.value);
    let query_expr = el.args.iter().find(|a| a.key == "query").map(|a| &a.value);

    match (text_expr, query_expr) {
        (Some(text), None) => quote! { <span>{#text}</span> },
        (Some(text), Some(query)) => {
            let hl_id = next_extract_id();
            let hl_name = quote::format_ident!("__quoin_highlight_{}", hl_id);
            _bindings.push(quote! {
                let #hl_name = {
                    let __text_val = (#text).clone();
                    let __query_val = (#query).clone();
                    move || {
                        if __query_val.is_empty() {
                            return { use leptos::prelude::*; leptos::view! { <span>{__text_val.clone()}</span> } }.into_any();
                        }
                        let mut __parts: Vec<leptos::prelude::AnyView> = Vec::new();
                        let mut __remaining = __text_val.as_str();
                        let __query_lower = __query_val.to_lowercase();
                        while let Some(__idx) = __remaining.to_lowercase().find(&__query_lower) {
                            if __idx > 0 {
                                let __before: &str = &__remaining[..__idx];
                                __parts.push({ use leptos::prelude::*; leptos::view! { <span>{__before}</span> } }.into_any());
                            }
                            let __match: &str = &__remaining[__idx..__idx + __query_val.len()];
                            __parts.push(
                                { use leptos::prelude::*; leptos::view! { <span class="bg-yellow-200 text-black">{__match}</span> } }.into_any()
                            );
                            __remaining = &__remaining[__idx + __query_val.len()..];
                        }
                        if !__remaining.is_empty() {
                            __parts.push({ use leptos::prelude::*; leptos::view! { <span>{__remaining}</span> } }.into_any());
                        }
                        { use leptos::prelude::*; leptos::view! { <span>{__parts.into_iter()}</span> } }.into_any()
                    }
                };
            });
            quote! { {#hl_name()} }
        }
        (None, _) => quote! { <span></span> },
    }
}

// ---------------------------------------------------------------------------
// Clipboard button
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
    emit_html_tag_inner(el, tag, bindings, inside_for)
}

fn emit_html_tag_inner(
    el: &Element,
    tag: &str,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let mut attrs = Vec::new();

    let has_value = el.args.iter().any(|a| a.key == "value");
    let has_on_input = el.args.iter().any(|a| a.key == "on_input");
    let auto_bind_input = tag == "input" && has_value && !has_on_input;

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
                    attrs.push(quote! { prop:value={
                        let __val = (#value).clone();
                        move || __val.get()
                    }});
                } else {
                    attrs.push(quote! { value={#value} });
                }
            }
            _ => {}
        }
    }

    if auto_bind_input {
        let value_expr = el
            .args
            .iter()
            .find(|a| a.key == "value")
            .map(|a| &a.value)
            .unwrap();
        let bind_id = next_extract_id();
        let bind_name = quote::format_ident!("__quoin_input_bind_{}", bind_id);
        bindings.push(quote! {
            let #bind_name = {
                let __signal = (#value_expr).clone();
                move |ev: leptos::ev::Event| {
                    __signal.set(leptos::prelude::event_target_value(&ev));
                }
            };
        });
        attrs.push(quote! { on:input=#bind_name });
    }

    let mut children = Vec::new();
    if let Some(children_expr) = &el.children_expr {
        children.push(quote! {
            {#children_expr.into_iter().map(|v| v.into_any()).collect::<Vec<_>>()}
        });
    } else {
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }
    }

    let tag_ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
    let is_void = matches!(tag, "input" | "hr" | "br" | "img");
    if is_void {
        quote! { <#tag_ident #(#attrs)* /> }
    } else if children.is_empty() {
        quote! { <#tag_ident #(#attrs)*></#tag_ident> }
    } else {
        quote! { <#tag_ident #(#attrs)*> #(#children)* </#tag_ident> }
    }
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

fn emit_tabs(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_tabs_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_tabs_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_tabs_plain(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
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

    let on_click_wrapped = wrap_event_handler(on_click_expr);

    let tab_labels: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "tab"
            {
                let tab_label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;

                let param_shadows: Vec<TokenStream> = param_idents
                    .iter()
                    .map(|id| quote! { let #id = #index; })
                    .collect();
                let call_args: Vec<TokenStream> =
                    param_idents.iter().map(|id| quote! { #id }).collect();

                Some(quote! {
                    <li
                        class={move || if #index == #active_expr { "active" } else { "" }}
                        on:click={
                            #(#param_shadows)*
                            let __tab_on_click = #on_click_wrapped;
                            move |_| { __tab_on_click(#(#call_args)*) }
                        }
                    >{#tab_label}</li>
                })
            } else {
                None
            }
        })
        .collect();

    quote! { <ul class="tabs"> #(#tab_labels)* </ul> }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_tabs_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
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

    let on_click_wrapped = wrap_event_handler(on_click_expr);

    let tabs_alias = quote::format_ident!("Tabs_{}", next_extract_id());
    let tabs_list_alias = quote::format_ident!("TabsList_{}", next_extract_id());
    let tabs_trigger_alias = quote::format_ident!("TabsTrigger_{}", next_extract_id());

    bindings.push(quote! {
        let #tabs_alias = leptos_shadcn_ui::Tabs;
        let #tabs_list_alias = leptos_shadcn_ui::TabsList;
        let #tabs_trigger_alias = leptos_shadcn_ui::TabsTrigger;
    });

    let tab_triggers: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "tab"
            {
                let tab_label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let index = e.args.iter().find(|a| a.key == "index").map(|a| &a.value)?;
                Some(quote! {
                    <#tabs_trigger_alias value={#index.to_string()} class="text-white">{#tab_label}</#tabs_trigger_alias>
                })
            } else {
                None
            }
        })
        .collect();

    quote! {
        <#tabs_alias
            value={leptos::prelude::Signal::derive(move || (#active_expr).to_string())}
            on_value_change={
                let __on_click = #on_click_wrapped;
                move |val: String| {
                    if let Ok(idx) = val.parse::<usize>() {
                        __on_click(idx);
                    }
                }
            }
        >
            <#tabs_list_alias>
                #(#tab_triggers)*
            </#tabs_list_alias>
        </#tabs_alias>
    }
}

// ---------------------------------------------------------------------------
// Dropdown menu
// ---------------------------------------------------------------------------
fn emit_dropdown_menu(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_dropdown_menu_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_dropdown_menu_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_dropdown_menu_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let trigger_expr = match &el.trigger_expr {
        Some(e) => e,
        None => return quote! { <div>dropdown: missing trigger</div> },
    };

    let dd_id = next_extract_id();
    let open_name = quote::format_ident!("__quoin_dd_open_{}", dd_id);
    let node_ref_name = quote::format_ident!("__quoin_dd_ref_{}", dd_id);
    bindings.push(quote! {
        let #open_name = leptos::prelude::create_signal(false);
        let #node_ref_name = leptos::prelude::create_node_ref::<html::Div>();
    });

    let item_tokens: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "item"
            {
                let item_label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let on_click = e
                    .args
                    .iter()
                    .find(|a| a.key == "on_click")
                    .map(|a| &a.value)?;
                let checked = e.args.iter().any(|a| a.key == "checked" && {
                    matches!(&a.value, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) if b.value)
                });
                let handler = wrap_event_handler(on_click);
                let close_open = quote! { #open_name.set(false); };
                let checked_icon = if checked {
                    Some(quote! { <span class="mr-2">"✓"</span> })
                } else {
                    None
                };
                Some(quote! {
                    <div
                        class="px-3 py-2 cursor-pointer text-white hover:bg-gray-600 flex items-center"
                        on:click={
                            let __item_handler = #handler;
                            move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                                #close_open;
                                __item_handler(ev);
                            }
                        }
                    >{#checked_icon}{#item_label}</div>
                })
            } else {
                None
            }
        })
        .collect();

    let trigger_inner = emit_node(
        &RenderNode::Expr(trigger_expr.clone()),
        bindings,
        inside_for,
    );

    quote! {
        <div
            node_ref=#node_ref_name
            class="relative inline-block"
            on:click=move |ev: leptos::ev::MouseEvent| {
                ev.stop_propagation();
                #open_name.update(|v| *v = !*v);
            }
        >
            {#trigger_inner}
            {
                move || #open_name.get().then(|| {
                    leptos::view! {
                        <div
                            class="absolute top-full left-0 z-50 min-w-32 rounded-md border border-gray-700 bg-gray-800 py-1 shadow-lg"
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                            }
                            on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                ev.prevent_default();
                            }
                        >
                            #(#item_tokens)*
                        </div>
                    }.into_any()
                })
            }
        </div>
    }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_dropdown_menu_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let trigger_expr = match &el.trigger_expr {
        Some(e) => e,
        None => return quote! { <div>dropdown: missing trigger</div> },
    };

    let dm_alias = quote::format_ident!("DropdownMenu_{}", next_extract_id());
    let dmt_alias = quote::format_ident!("DropdownMenuTrigger_{}", next_extract_id());
    let dmc_alias = quote::format_ident!("DropdownMenuContent_{}", next_extract_id());
    let dmi_alias = quote::format_ident!("DropdownMenuItem_{}", next_extract_id());

    bindings.push(quote! {
        let #dm_alias = leptos_shadcn_ui::DropdownMenu;
        let #dmt_alias = leptos_shadcn_ui::DropdownMenuTrigger;
        let #dmc_alias = leptos_shadcn_ui::DropdownMenuContent;
        let #dmi_alias = leptos_shadcn_ui::DropdownMenuItem;
    });

    let item_tokens: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "item"
            {
                let item_label = e.args.iter().find(|a| a.key == "label").map(|a| &a.value)?;
                let on_click = e
                    .args
                    .iter()
                    .find(|a| a.key == "on_click")
                    .map(|a| &a.value)?;
                let handler = wrap_event_handler(on_click);
                Some(quote! {
                    <#dmi_alias on_click={#handler}>
                        {#item_label}
                    </#dmi_alias>
                })
            } else {
                None
            }
        })
        .collect();

    let trigger_inner = emit_node(
        &RenderNode::Expr(trigger_expr.clone()),
        bindings,
        inside_for,
    );

    quote! {
        <#dm_alias>
            <#dmt_alias>
                {#trigger_inner}
            </#dmt_alias>
            <#dmc_alias>
                #(#item_tokens)*
            </#dmc_alias>
        </#dm_alias>
    }
}

// ---------------------------------------------------------------------------
// Data table
// ---------------------------------------------------------------------------
fn emit_data_table(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    #[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
    {
        emit_data_table_shadcn(el, bindings, inside_for)
    }
    #[cfg(not(all(feature = "leptos", feature = "leptos-shadcn")))]
    {
        emit_data_table_plain(el, bindings, inside_for)
    }
}

#[allow(dead_code)]
fn emit_data_table_plain(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let rows_expr = el.args.iter().find(|a| a.key == "rows").map(|a| &a.value);
    let striped = find_arg_bool(el, "striped");
    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c
            && e.name == "column"
        {
            let col_label = e
                .args
                .iter()
                .find(|a| a.key == "label")
                .map(|a| &a.value)
                .unwrap_or(&empty_label);
            let width = e.args.iter().find(|a| a.key == "width").map(|a| &a.value);
            let mut th_attrs = vec![quote! { class="px-3 py-2 text-gray-400 font-medium" }];
            if let Some(w) = width {
                th_attrs.push(quote! { style=format!("width: {}px", #w) });
            }
            header_cells.push(quote! { <th #(#th_attrs)*>{#col_label}</th> });

            let render_closure = e.args.iter().find(|a| a.key == "render").map(|a| &a.value);
            if let Some(rc) = render_closure {
                let col_id = next_extract_id();
                let render_name = quote::format_ident!("__quoin_col_{}", col_id);
                bindings.push(quote! { let #render_name = std::sync::Arc::new(#rc); });
                let mut td_attrs = vec![quote! { class="px-3 py-2 text-white" }];
                if let Some(w) = width {
                    td_attrs.push(quote! { style=format!("width: {}px", #w) });
                }
                row_cells.push(quote! { <td #(#td_attrs)*>{ (&*#render_name)(&__row) }</td> });
            } else {
                row_cells.push(quote! { <td class="px-3 py-2 text-white"></td> });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let striped_class = if striped { " table-striped" } else { "" };

    // Direct block evaluation avoids FnOnce/FnMut and PartialEq bounds entirely.
    // It perfectly mirrors how `emit_for_inner` operates, rebuilding the rows
    // on every render cycle naturally via Leptos's reactive graph.
    quote! {
        <table class={concat!("w-full", #striped_class)}>
            <thead><tr> #(#header_cells)* </tr></thead>
            <tbody>
                {
                    let __rows = (#rows).clone();
                    __rows.into_iter().map(|__row| {
                        leptos::view! { <tr> #(#row_cells)* </tr> }
                    }).collect::<Vec<_>>()
                }
            </tbody>
        </table>
    }
}

#[cfg(all(feature = "leptos", feature = "leptos-shadcn"))]
fn emit_data_table_shadcn(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    _inside_for: bool,
) -> TokenStream {
    let rows_expr = el.args.iter().find(|a| a.key == "rows").map(|a| &a.value);
    let striped = find_arg_bool(el, "striped");
    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c
            && e.name == "column"
        {
            let col_label = e
                .args
                .iter()
                .find(|a| a.key == "label")
                .map(|a| &a.value)
                .unwrap_or(&empty_label);

            header_cells.push(quote! {
                <th class="px-3 py-2 text-gray-400 font-medium">{#col_label}</th>
            });

            let render_closure = e.args.iter().find(|a| a.key == "render").map(|a| &a.value);
            if let Some(rc) = render_closure {
                let col_id = next_extract_id();
                let render_name = quote::format_ident!("__quoin_col_{}", col_id);
                bindings.push(quote! { let #render_name = std::sync::Arc::new(#rc); });
                row_cells.push(quote! {
                    <td class="px-3 py-2 text-white">{ (&*#render_name)(&__row) }</td>
                });
            } else {
                row_cells.push(quote! {
                    <td class="px-3 py-2 text-white"></td>
                });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let class_value = if striped {
        "w-full table-striped"
    } else {
        "w-full"
    };

    let table_alias = quote::format_ident!("Table_{}", next_extract_id());
    bindings.push(quote! {
        let #table_alias = leptos_shadcn_ui::Table;
    });

    // Direct block evaluation avoids FnOnce/FnMut and PartialEq bounds entirely.
    // It perfectly mirrors how `emit_for_inner` operates, rebuilding the rows
    // on every render cycle naturally via Leptos's reactive graph.
    quote! {
        <#table_alias class=#class_value>
            <thead><tr>#(#header_cells)*</tr></thead>
            <tbody>
                {
                    let __rows = (#rows).clone();
                    __rows.into_iter().map(|__row| {
                        leptos::view! { <tr> #(#row_cells)* </tr> }
                    }).collect::<Vec<_>>()
                }
            </tbody>
        </#table_alias>
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

fn find_arg_bool(el: &Element, key: &str) -> bool {
    el.args
        .iter()
        .find(|a| a.key == key)
        .map(|a| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(b),
                ..
            }) = &a.value
            {
                return b.value;
            }
            false
        })
        .unwrap_or(false)
}

fn find_arg_string(el: &Element, key: &str) -> Option<String> {
    el.args.iter().find(|a| a.key == key).and_then(|a| {
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
}
