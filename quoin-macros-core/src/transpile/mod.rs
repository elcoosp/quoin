//! Transpilation utilities for `quoin_render!`.
//!
//! This module contains helpers and framework-specific code generators that
//! transform high-level quoin DSL constructs into framework-native code.
//!
//! # Submodule Overview
//!
//! | Submodule              | Description |
//!------------------------|-------------|
//! [`tailwind`]           | Transpiles Tailwind CSS class strings into GPUI builder-method chains. |
//! [`table_codegen`]      | Generates data-table render code for each framework. |
//! [`dropdown_codegen`]   | Generates dropdown menu code for each framework. |
//! [`virtual_list_codegen`]| Generates virtual list code (currently falls back to scrollable div). |
//! [`rich_text_codegen`]  | Generates rich text / styled text render code. |
//! [`icon_codegen`]       | Maps icon names to inline SVG token streams. |
//! [`theme_tokens`]       | Maps theme token names to Tailwind CSS classes. |
//!
//! # Shared Utilities
//!
//! The module also provides AST visitor helpers used across all emitters:
//!
//! - [`collect_handler_idents`] — collects single-segment path idents from a closure body.
//! - [`collect_block_idents`] — collects idents from a block, including nested closures.
//! - [`collect_handler_idents_excluding_params`] — like above but filters out closure params.
//! - [`force_move_on_closure`] — ensures the outermost closure has the `move` keyword.

pub mod dropdown_codegen;
pub mod icon_codegen;
pub mod rich_text_codegen;
pub mod table_codegen;
pub mod tailwind;
pub mod theme_tokens;
pub mod virtual_list_codegen;

use syn::visit::Visit;
use syn::visit_mut::VisitMut;

/// Collect all single-segment path idents from an expression, skipping nested closures.
pub fn collect_handler_idents(expr: &syn::Expr) -> Vec<proc_macro2::Ident> {
    let body_expr: &syn::Expr = match expr {
        syn::Expr::Closure(c) => &c.body,
        other => other,
    };

    let mut collector = PathIdentCollectorSkipClosures(vec![]);
    collector.visit_expr(body_expr);
    collector.0.sort_by_key(|id| id.to_string());
    collector.0.dedup_by(|a, b| *a == *b);
    collector.0
}

/// Collect all single-segment path idents from a block, including nested closures.
pub fn collect_block_idents(block: &syn::Block) -> Vec<proc_macro2::Ident> {
    let mut collector = PathIdentCollectorAll(vec![]);
    collector.visit_block(block);
    collector.0.sort_by_key(|id| id.to_string());
    collector.0.dedup_by(|a, b| *a == *b);
    collector.0
}

/// Ensure the outermost closure has `move`. Non-closure exprs are returned unchanged.
pub fn force_move_on_closure(expr: &syn::Expr) -> syn::Expr {
    struct ForceMove;
    impl VisitMut for ForceMove {
        fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
            closure.capture = Some(syn::Token![move](proc_macro2::Span::call_site()));
        }
    }
    let mut expr = expr.clone();
    ForceMove.visit_expr_mut(&mut expr);
    expr
}

pub fn collect_handler_idents_excluding_params(expr: &syn::Expr) -> Vec<proc_macro2::Ident> {
    let param_idents: std::collections::HashSet<String> = match expr {
        syn::Expr::Closure(c) => c
            .inputs
            .iter()
            .filter_map(|pat| {
                if let syn::Pat::Ident(pi) = pat {
                    Some(pi.ident.to_string())
                } else if let syn::Pat::Type(pt) = pat {
                    if let syn::Pat::Ident(pi) = pt.pat.as_ref() {
                        Some(pi.ident.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect(),
        _ => std::collections::HashSet::new(),
    };

    collect_handler_idents(expr)
        .into_iter()
        .filter(|id| !param_idents.contains(&id.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Visitors
// ---------------------------------------------------------------------------

struct PathIdentCollectorSkipClosures(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorSkipClosures {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        if expr_path.path.segments.len() == 1
            && expr_path.path.leading_colon.is_none()
            && let Some(seg) = expr_path.path.segments.last()
        {
            self.0.push(seg.ident.clone());
        }
        syn::visit::visit_expr_path(self, expr_path);
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}
}

struct PathIdentCollectorAll(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorAll {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        if expr_path.path.segments.len() == 1
            && expr_path.path.leading_colon.is_none()
            && let Some(seg) = expr_path.path.segments.last()
        {
            self.0.push(seg.ident.clone());
        }
        syn::visit::visit_expr_path(self, expr_path);
    }
}

/// Collect all single-segment path idents referenced in a slice of RenderNodes.

/// Collect all single-segment path idents referenced in a slice of RenderNodes (fully recursive).
pub fn collect_render_idents(nodes: &[crate::render_ast::RenderNode]) -> Vec<proc_macro2::Ident> {
    use crate::render_ast::RenderNode;
    use syn::visit::Visit;
    struct IdentCollector(Vec<proc_macro2::Ident>);
    impl Visit<'_> for IdentCollector {
        fn visit_expr_path(&mut self, expr_path: &syn::ExprPath) {
            if expr_path.path.segments.len() == 1 && expr_path.path.leading_colon.is_none() {
                if let Some(seg) = expr_path.path.segments.last() {
                    self.0.push(seg.ident.clone());
                }
            }
            syn::visit::visit_expr_path(self, expr_path);
        }
        fn visit_expr_closure(&mut self, _: &syn::ExprClosure) {}
    }

    let mut collector = IdentCollector(vec![]);
    fn walk(nodes: &[RenderNode], collector: &mut IdentCollector) {
        for node in nodes {
            match node {
                RenderNode::Element(el) => {
                    for arg in &el.args {
                        collector.visit_expr(&arg.value);
                    }
                    walk(&el.children, collector);
                }
                RenderNode::Expr(e) => {
                    collector.visit_expr(e);
                }
                RenderNode::If(if_node) => {
                    collector.visit_expr(&if_node.condition);
                    walk(&if_node.then_branch, collector);
                    if let Some(else_branch) = &if_node.else_branch {
                        walk(else_branch, collector);
                    }
                }
                RenderNode::For(for_node) => {
                    // Only collect from the iterable expression — the loop body
                    // is a separate scope whose variables are already handled
                    // by emit_for_inner. Cloning them in the outer scope would
                    // try to resolve identifiers that do not exist yet.
                    collector.visit_expr(&for_node.iterable);
                }
                _ => {}
            }
        }
    }
    walk(nodes, &mut collector);
    collector.0.sort_by_key(|id| id.to_string());
    collector.0.dedup_by(|a, b| *a == *b);
    collector.0
}
