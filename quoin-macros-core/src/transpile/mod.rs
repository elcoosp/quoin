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
//! [`tailwind_grsx`]      | `gpui-rsx-classes` engine: gpui-rsx-backed lowering for classes quoin does not know, dynamic classes, and `rsx.toml` themes. |
//! [`icon_codegen`]       | Maps icon names to inline SVG token streams. |
//! [`theme_tokens`]       | Maps theme token names to Tailwind CSS classes. |
//!
//! The per-framework render code for `data_table`, `dropdown_menu`,
//! `virtual_list` and rich/styled text lives in the backend emitters
//! (`emit::render_*`); the former shared `*_codegen` modules here were
//! superseded duplicates and were removed (ROADMAP Phase 1.8 follow-up:
//! "wire or delete the four orphaned `*_codegen` modules" — deleted).
//!
//! # Shared Utilities
//!
//! The module also provides AST visitor helpers used across all emitters:
//!
//! - [`collect_handler_idents`] — collects single-segment path idents from a closure body.
//! - [`collect_block_idents`] — collects idents from a block, including nested closures.
//! - [`collect_handler_idents_excluding_params`] — like above but filters out closure params.
//! - [`force_move_on_closure`] — ensures the outermost closure has the `move` keyword.

pub mod icon_codegen;
pub mod tailwind;
// The gpui-rsx engine lowers classes only for the GPUI backend, so it is
// compiled when both switches are on. `gpui-rsx-classes` alone (without
// `gpui`) changes nothing — there is no GPUI emitter to route through.
#[cfg(all(feature = "gpui", feature = "gpui-rsx-classes"))]
pub mod tailwind_grsx;
pub mod theme_tokens;

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
    collector.0.sort_by_key(std::string::ToString::to_string);
    collector.0.dedup_by(|a, b| *a == *b);
    collector.0
}

/// Collect all single-segment path idents from a block, including nested closures.
pub fn collect_block_idents(block: &syn::Block) -> Vec<proc_macro2::Ident> {
    let mut collector = PathIdentCollectorAll(vec![]);
    collector.visit_block(block);
    collector.0.sort_by_key(std::string::ToString::to_string);
    collector.0.dedup_by(|a, b| *a == *b);
    collector.0
}

/// Ensure the outermost closure has `move`. Non-closure exprs are returned unchanged.
#[must_use]
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

#[must_use]
/// Collects identifiers used inside `expr` that are *not* closure params and
/// *not* declared by a `let` binding inside the handler body — used to detect
/// which reactive signals a handler touches.
///
/// The `let` filter matters: shadowing a handler-local binding with an outer
/// `let x = x.clone();` would reference a variable that does not exist in the
/// enclosing scope and fail to compile.
pub fn collect_handler_idents_excluding_params(expr: &syn::Expr) -> Vec<proc_macro2::Ident> {
    let excluded: std::collections::HashSet<String> = match expr {
        syn::Expr::Closure(c) => c
            .inputs
            .iter()
            .filter_map(pattern_ident)
            .chain(
                declared_idents(&c.body)
                    .into_iter()
                    .map(|id| id.to_string()),
            )
            .collect(),
        _ => std::collections::HashSet::new(),
    };

    collect_handler_idents(expr)
        .into_iter()
        .filter(|id| !excluded.contains(&id.to_string()))
        .collect()
}

/// The plain identifier of a closure parameter pattern, if any.
fn pattern_ident(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(pi) => Some(pi.ident.to_string()),
        syn::Pat::Type(pt) => pattern_ident(&pt.pat),
        _ => None,
    }
}

/// Every identifier a pattern binds (`let (a, b) = ..`, `let Some(x) = ..`).
fn pattern_binding_idents(pat: &syn::Pat, out: &mut Vec<proc_macro2::Ident>) {
    match pat {
        syn::Pat::Ident(pi) => out.push(pi.ident.clone()),
        syn::Pat::Type(pt) => pattern_binding_idents(&pt.pat, out),
        syn::Pat::Tuple(t) => {
            for elem in &t.elems {
                pattern_binding_idents(elem, out);
            }
        }
        syn::Pat::TupleStruct(ts) => {
            for elem in &ts.elems {
                pattern_binding_idents(elem, out);
            }
        }
        syn::Pat::Struct(ps) => {
            for field in &ps.fields {
                pattern_binding_idents(&field.pat, out);
            }
        }
        syn::Pat::Reference(r) => pattern_binding_idents(&r.pat, out),
        syn::Pat::Or(or) => {
            // Only the first alternative's bindings are safe to name; OR
            // patterns that bind different names are rejected by rustc long
            // before our shadow would matter.
            if let Some(first) = or.cases.first() {
                pattern_binding_idents(first, out);
            }
        }
        _ => {}
    }
}

/// Identifiers declared by `let` statements anywhere inside the block,
/// including nested blocks that are still in the handler's own scope chain.
fn declared_idents(body: &syn::Expr) -> Vec<proc_macro2::Ident> {
    struct LetDeclCollector(Vec<proc_macro2::Ident>);

    impl<'ast> Visit<'ast> for LetDeclCollector {
        fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
            if let syn::Stmt::Local(local) = stmt {
                pattern_binding_idents(&local.pat, &mut self.0);
            }
            syn::visit::visit_stmt(self, stmt);
        }

        // Closures introduce a new scope: their `let`s do not leak into the
        // enclosing handler, so do not descend.
        fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}
    }

    let mut collector = LetDeclCollector(vec![]);
    collector.visit_expr(body);
    collector.0
}

// ---------------------------------------------------------------------------
// Visitors
// ---------------------------------------------------------------------------

struct PathIdentCollectorSkipClosures(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorSkipClosures {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        let segments: Vec<_> = expr_path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.clone())
            .collect();
        if expr_path.path.leading_colon.is_none() {
            match segments.as_slice() {
                [single] => self.0.push(single.clone()),
                // `store.draft` reads a field off a handle: the *root* is the
                // captured variable. Rust naming puts types/consts in
                // UpperCamel, so only lowercase roots are treated as
                // variables (`Filter::Active` must not shadow the enum);
                // the `capture:` argument is the documented escape hatch.
                [root, ..]
                    if root
                        .to_string()
                        .starts_with(|c: char| c.is_ascii_lowercase()) =>
                {
                    self.0.push(root.clone());
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_path(self, expr_path);
    }

    fn visit_expr_closure(&mut self, _node: &'ast syn::ExprClosure) {}

    /// The callee of a call is a function name, not a captured variable
    /// (`derived(&sig, ..)` must not shadow a variable named `derived`).
    /// Only the arguments are captured.
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        for arg in &call.args {
            self.visit_expr(arg);
        }
    }

    /// `syn` does not recurse into macro bodies (they are raw token trees,
    /// not parsed expressions), so a signal referenced only inside
    /// `format!("Count: {}", count.get())` would otherwise be invisible to
    /// the capture analysis and moved into the closure instead of cloned.
    ///
    /// The tokens are first parsed as a comma-separated expression list
    /// (the shape of `format!`/`println!`/`assert!` arguments) and visited
    /// as real expressions, so a nested closure's *parameters* (`format!`
    /// arg `items.iter().filter(|t| …)`) are recognized and excluded
    /// instead of being mistaken for captured variables. Macro bodies that
    /// are not an expression list fall back to the raw token scan.
    fn visit_expr_macro(&mut self, mac: &'ast syn::ExprMacro) {
        use syn::parse::Parser as _;
        let parser = |input: syn::parse::ParseStream| {
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(input)
        };
        if let Ok(args) = parser.parse2(mac.mac.tokens.clone()) {
            for arg in &args {
                self.visit_expr(arg);
            }
            return;
        }
        self.scan_token_stream(mac.mac.tokens.clone());
    }
}

/// Rust keywords that can appear as bare idents in macro bodies but never
/// name a captured variable.
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "async", "await", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

impl PathIdentCollectorSkipClosures {
    /// A small token-tree walker: collects single-segment idents that are
    /// not function/macro names, not path segments, and not field accesses.
    /// Concretely an ident is skipped when it
    ///
    /// - is a keyword,
    /// - is followed by `!` (macro name) or `::` (path prefix),
    /// - is followed by a parenthesis group (a call — in a raw token stream
    ///   the `()` of `x.get()` is a `Group(Parenthesis)`, not a `(` punct),
    /// - is preceded by `.` (field access: `counter.count` must not shadow
    ///   a variable named `count`) or `::` (path suffix).
    fn scan_token_stream(&mut self, stream: proc_macro2::TokenStream) {
        let trees: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();
        for (index, tree) in trees.iter().enumerate() {
            match tree {
                proc_macro2::TokenTree::Group(group) => self.scan_token_stream(group.stream()),
                proc_macro2::TokenTree::Ident(ident) => {
                    let name = ident.to_string();
                    if KEYWORDS.contains(&name.as_str()) {
                        continue;
                    }
                    let next_is_special = trees.get(index + 1).is_some_and(|next| match next {
                        proc_macro2::TokenTree::Punct(p) => {
                            p.as_char() == '!' || p.as_char() == ':' || p.as_char() == '('
                        }
                        proc_macro2::TokenTree::Group(g) => {
                            g.delimiter() == proc_macro2::Delimiter::Parenthesis
                        }
                        _ => false,
                    });
                    let prev_is_accessor = index > 0
                        && matches!(
                            &trees[index - 1],
                            proc_macro2::TokenTree::Punct(p)
                                if p.as_char() == '.' || p.as_char() == ':'
                        );
                    if next_is_special || prev_is_accessor {
                        continue;
                    }
                    // Field-chain roots: `counter.count.get()` inside a
                    // `format!` body captures `counter`, not the fields.
                    let follows_field =
                        index > 0 && matches!(&trees[index - 1], proc_macro2::TokenTree::Ident(_));
                    if follows_field
                        && ident
                            .to_string()
                            .starts_with(|c: char| c.is_ascii_uppercase())
                    {
                        continue;
                    }
                    self.0.push(ident.clone());
                }
                _ => {}
            }
        }
    }
}

struct PathIdentCollectorAll(Vec<proc_macro2::Ident>);

impl<'ast> Visit<'ast> for PathIdentCollectorAll {
    fn visit_expr_path(&mut self, expr_path: &'ast syn::ExprPath) {
        let segments: Vec<_> = expr_path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.clone())
            .collect();
        if expr_path.path.leading_colon.is_none() {
            match segments.as_slice() {
                [single] => self.0.push(single.clone()),
                // `store.draft` reads a field off a handle: the *root* is the
                // captured variable. Rust naming puts types/consts in
                // UpperCamel, so only lowercase roots are treated as
                // variables (`Filter::Active` must not shadow the enum);
                // the `capture:` argument is the documented escape hatch.
                [root, ..]
                    if root
                        .to_string()
                        .starts_with(|c: char| c.is_ascii_lowercase()) =>
                {
                    self.0.push(root.clone());
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_path(self, expr_path);
    }
}

/// Collects the outer variables captured anywhere inside a render subtree,
/// used to place per-closure shadow clones *outside* each emitted `move`
/// closure (`Fn` closures cannot capture by value, so every closure site
/// needs its own clones hoisted into an enclosing block).
pub fn collect_node_captures<S: std::hash::BuildHasher>(
    nodes: &[crate::render_ast::RenderNode],
    loop_pats: &mut std::collections::HashSet<String, S>,
) -> Vec<proc_macro2::Ident> {
    use crate::render_ast::RenderNode;

    fn walk<S: std::hash::BuildHasher>(
        node: &RenderNode,
        out: &mut Vec<proc_macro2::Ident>,
        loop_pats: &mut std::collections::HashSet<String, S>,
    ) {
        match node {
            RenderNode::Element(el) => {
                for arg in &el.args {
                    // Skip string-literal args (classes, placeholders): no
                    // captures there.
                    if matches!(&arg.value, syn::Expr::Lit(_)) {
                        continue;
                    }
                    out.extend(collect_handler_idents_excluding_params(&arg.value));
                }
                if let Some(children) = &el.children_expr {
                    out.extend(collect_handler_idents_excluding_params(children));
                }
                if let Some(trigger) = &el.trigger_expr {
                    out.extend(collect_handler_idents_excluding_params(trigger));
                }
                for child in &el.children {
                    walk(child, out, loop_pats);
                }
            }
            RenderNode::Expr(expr) => out.extend(collect_handler_idents_excluding_params(expr)),
            RenderNode::If(if_node) => {
                out.extend(collect_handler_idents_excluding_params(&if_node.condition));
                for child in &if_node.then_branch {
                    walk(child, out, loop_pats);
                }
                if let Some(else_nodes) = &if_node.else_branch {
                    for child in else_nodes {
                        walk(child, out, loop_pats);
                    }
                }
            }
            RenderNode::For(for_node) => {
                out.extend(collect_handler_idents_excluding_params(&for_node.iterable));
                if let Some(key) = &for_node.key {
                    out.extend(collect_handler_idents_excluding_params(key));
                }
                loop_pats.insert(for_node.pat.to_string());
                for child in &for_node.body {
                    walk(child, out, loop_pats);
                }
            }
            RenderNode::Text(_) | RenderNode::Root(_) => {}
        }
    }

    let mut out = Vec::new();
    for node in nodes {
        walk(node, &mut out, loop_pats);
    }
    out.retain(|id| !loop_pats.contains(&id.to_string()));
    out.sort_by_key(std::string::ToString::to_string);
    out.dedup_by(|a, b| *a == *b);
    out
}
