pub mod dropdown_codegen;
pub mod rich_text_codegen;
pub mod table_codegen;
pub mod tailwind;
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
    collector
        .0.sort_by_key(|a| a.to_string());
    collector.0.dedup_by(|a, b| *b == a.to_string());
    collector.0
}

/// Collect all single-segment path idents from a block, including nested closures.
pub fn collect_block_idents(block: &syn::Block) -> Vec<proc_macro2::Ident> {
    let mut collector = PathIdentCollectorAll(vec![]);
    collector.visit_block(block);
    collector
        .0.sort_by_key(|a| a.to_string());
    collector.0.dedup_by(|a, b| *b == a.to_string());
    collector.0
}

/// Ensure the outermost closure has `move`. Non-closure exprs are returned unchanged.
pub fn force_move_on_closure(expr: &syn::Expr) -> syn::Expr {
    struct ForceMove;
    impl VisitMut for ForceMove {
        fn visit_expr_closure_mut(&mut self, closure: &mut syn::ExprClosure) {
            closure.capture = Some(syn::Token![move](proc_macro2::Span::call_site()));
            // Do not recurse; only outermost needs move
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
        if expr_path.path.segments.len() == 1 && expr_path.path.leading_colon.is_none()
            && let Some(seg) = expr_path.path.segments.last() {
                self.0.push(seg.ident.clone());
            }
        syn::visit::visit_expr_path(self, expr_path);
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}
}

struct PathIdentCollectorAll(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorAll {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        if expr_path.path.segments.len() == 1 && expr_path.path.leading_colon.is_none()
            && let Some(seg) = expr_path.path.segments.last() {
                self.0.push(seg.ident.clone());
            }
        syn::visit::visit_expr_path(self, expr_path);
    }
}
