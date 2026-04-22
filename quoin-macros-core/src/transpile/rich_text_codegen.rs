//! Rich text / styled text code generation.
//!
//! Generates framework-specific code for rendering text with per-run styling
//! (e.g., colored substrings, mixed font weights). Each generator accepts:
//!
//! - `text_expr`: The base text expression.
//! - `base_color`: Optional default text color.
//! - `font_size`: Base font size in pixels.
//! - `runs_expr`: Optional expression yielding styled run descriptors.
//!
//! # Framework Output
//!
//! - **GPUI** ([`generate_gpui_rich_text`]): Uses `gpui_component::StyledText`
//!   with `.with_runs()` for per-run colors and backgrounds.
//! - **Leptos** ([`generate_leptos_rich_text`]): Emits nested `<span>` elements
//!   with inline `style` attributes for color and background.
//! - **Dioxus** ([`generate_dioxus_rich_text`]): Emits `rsx!` nested spans with
//!   inline styles.

#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;
#[cfg(feature = "gpui")]
pub fn generate_gpui_rich_text(
    text_expr: &syn::Expr,
    base_color: Option<&syn::Expr>,
    font_size: f32,
    runs_expr: Option<&syn::Expr>,
) -> TokenStream {
    let color_chain = if let Some(color) = base_color {
        quote! { .color(#color) }
    } else {
        quote! {}
    };
    let runs_chain = if let Some(runs) = runs_expr {
        quote! {
            .with_runs(
                #runs.iter().map(|__run| {
                    gpui_component::text::TextStyle {
                        color: __run.fg_color,
                        background_color: Some(__run.bg_color),
                        font_weight: Some(gpui::FontWeight::MEDIUM),
                        font_size: Some(gpui::px(#font_size)),
                        ..Default::default()
                    }.to_run(__run.len)
                }).collect::<Vec<_>>()
            )
        }
    } else {
        quote! {}
    };
    quote! {
        gpui_component::StyledText::new(#text_expr.clone())
            #color_chain
            .text_size(gpui::px(#font_size))
            #runs_chain
    }
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_rich_text(
    text_expr: &syn::Expr,
    base_color: Option<&syn::Expr>,
    font_size: f32,
    runs_expr: Option<&syn::Expr>,
) -> TokenStream {
    let style = format!("font-size: {}px;", font_size);
    if let Some(runs) = runs_expr {
        quote! {
            <span style=#style>
                {#runs.iter().map(|run| {
                    leptos::prelude::view! {
                        <span style:color=run.fg_color style:background-color=run.bg_color>
                            {&run.text}
                        </span>
                    }
                }).collect::<Vec<_>>()}
            </span>
        }
    } else {
        let color_attr = if let Some(color) = base_color {
            quote! { style:color=#color }
        } else {
            quote! {}
        };
        quote! {
            <span style=#style #color_attr>
                {#text_expr}
            </span>
        }
    }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_rich_text(
    _text_expr: &syn::Expr, // Prefixed with _ to avoid unused variable warning when runs_expr is Some
    base_color: Option<&syn::Expr>,
    font_size: f32,
    runs_expr: Option<&syn::Expr>,
) -> TokenStream {
    let style = format!("font-size: {}px;", font_size);
    if let Some(runs) = runs_expr {
        quote! {
            span { style: #style,
                #runs.iter().map(|run| {
                    rsx! {
                        span {
                            style: "color: {run.fg_color}; background-color: {run.bg_color}",
                            "{run.text}"
                        }
                    }
                })
            }
        }
    } else {
        let color_attr = if let Some(color) = base_color {
            quote! { color: #color }
        } else {
            quote! {}
        };
        quote! {
            span { style: #style, #color_attr, "{#_text_expr}" }
        }
    }
}
