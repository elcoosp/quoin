pub mod tailwind;
pub mod table_codegen;
pub mod virtual_list_codegen;
pub mod rich_text_codegen;
pub mod dropdown_codegen;

use syn::visit::Visit;
use syn::visit_mut::VisitMut;

/// Collect all single-segment path idents from an expression, skipping nested
/// closures (they have their own capture scope).
pub fn collect_handler_idents(expr: &syn::Expr) -> Vec<proc_macro2::Ident> {
    let body_expr: &syn::Expr = match expr {
        syn::Expr::Closure(c) => &c.body,
        other => other,
    };

    let mut collector = PathIdentCollectorSkipClosures(vec![]);
    collector.visit_expr(body_expr);
    collector
        .0
        .sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    collector.0.dedup_by(|a, b| a.to_string() == b.to_string());
    collector.0
}

/// Collect all single-segment path idents from a block, **including** those
/// inside nested closures. Used for component action blocks where the entire
/// block becomes the body of a `move ||` closure and every referenced ident
/// must be shadow-cloned beforehand.
pub fn collect_block_idents(block: &syn::Block) -> Vec<proc_macro2::Ident> {
    let mut collector = PathIdentCollectorAll(vec![]);
    collector.visit_block(block);
    collector
        .0
        .sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    collector.0.dedup_by(|a, b| a.to_string() == b.to_string());
    collector.0
}

/// Strip the `move` keyword from a closure expression so the emitter can
/// re-wrap it with explicit shadow-clones before the `move`.
pub fn strip_move_from_closure(expr: &syn::Expr) -> syn::Expr {
    struct StripMove;
    impl VisitMut for StripMove {
        fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
            closure.movability = None;
        }
    }
    let mut expr = expr.clone();
    StripMove.visit_expr_mut(&mut expr);
    expr
}

// ---------------------------------------------------------------------------
// Visitor that skips nested closures (for on_click / on_mouse_down handlers)
// ---------------------------------------------------------------------------

struct PathIdentCollectorSkipClosures(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorSkipClosures {
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

// ---------------------------------------------------------------------------
// Visitor that visits everything (for component action blocks)
// ---------------------------------------------------------------------------

struct PathIdentCollectorAll(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorAll {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        if expr_path.path.segments.len() == 1 && expr_path.path.leading_colon.is_none() {
            if let Some(seg) = expr_path.path.segments.last() {
                self.0.push(seg.ident.clone());
            }
        }
        syn::visit::visit_expr_path(self, expr_path);
    }
}
