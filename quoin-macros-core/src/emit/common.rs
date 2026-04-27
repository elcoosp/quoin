//! Shared utilities for render emitters.
//!
//! These functions reduce duplication across the GPUI, Leptos, and Dioxus
//! render backends. Currently used in `render_gpui.rs` and planned for use in
//! the other emitters during the ongoing deduplication refactor.

use crate::render_ast::{Element, RenderNode};
use syn::Expr;

/// Extract a boolean argument from an element's argument list.
#[allow(dead_code)]
pub fn find_arg_bool(el: &Element, key: &str) -> bool {
    el.args.iter().any(|a| {
        a.key == key
            && matches!(&a.value, Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(b), ..
            }) if b.value)
    })
}

/// Extract a string argument from an element's argument list.
#[allow(dead_code)]
pub fn find_arg_string(el: &Element, key: &str) -> Option<String> {
    el.args.iter().find(|a| a.key == key).and_then(|a| {
        if let Expr::Lit(expr_lit) = &a.value {
            if let syn::Lit::Str(s) = &expr_lit.lit {
                return Some(s.value());
            }
        }
        None
    })
}

/// Extract a floating‑point argument from an element's argument list.
#[allow(dead_code)]
pub fn find_arg_f32(el: &Element, key: &str) -> Option<f32> {
    el.args.iter().find(|a| a.key == key).and_then(|a| {
        if let Expr::Lit(syn::ExprLit { lit, .. }) = &a.value {
            match lit {
                syn::Lit::Float(f) => f.base10_parse::<f32>().ok(),
                syn::Lit::Int(i) => i.base10_parse::<f32>().ok(),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Extract a generic expression argument.
#[allow(dead_code)]
pub fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|a| a.key == key).map(|a| &a.value)
}

/// Extract tab definitions from children of a `tabs` element.
#[allow(dead_code)]
pub fn extract_tab_children(el: &Element) -> Vec<TabDefinition> {
    el.children.iter().filter_map(|c| {
        if let RenderNode::Element(e) = c && e.name == "tab" {
            let index = find_arg_expr(e, "index")?;
            let label = find_arg_string(e, "label")?;
            Some(TabDefinition { index: index.clone(), label })
        } else {
            None
        }
    }).collect()
}

/// Description of a single tab inside a `tabs` container.
#[derive(Debug, Clone)]
pub struct TabDefinition {
    pub index: Expr,
    pub label: String,
}
