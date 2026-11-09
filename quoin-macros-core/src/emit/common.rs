//! Shared utilities for render emitters.
//!
//! These helpers reduce duplication across the GPUI, Leptos, and Dioxus
//! render backends: every backend reads the same declarative-UI AST, so the
//! argument-extraction logic lives here exactly once.

use crate::parse::ComponentAst;
use crate::render_ast::Element;
use crate::transpile::collect_block_idents;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// Extract a boolean argument from an element's argument list.
#[must_use]
pub fn find_arg_bool(el: &Element, key: &str) -> bool {
    el.args.iter().any(|a| {
        a.key == key
            && matches!(&a.value, Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(b), ..
            }) if b.value)
    })
}

/// Extract a string argument from an element's argument list.
#[must_use]
pub fn find_arg_string(el: &Element, key: &str) -> Option<String> {
    el.args.iter().find(|a| a.key == key).and_then(|a| {
        if let Expr::Lit(expr_lit) = &a.value
            && let syn::Lit::Str(s) = &expr_lit.lit
        {
            return Some(s.value());
        }
        None
    })
}

/// Extract a floating‑point argument from an element's argument list.
#[must_use]
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
#[must_use]
pub fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Option<&'a Expr> {
    el.args.iter().find(|a| a.key == key).map(|a| &a.value)
}

/// Expands `component!` action methods into closure declarations that
/// shadow-clone the state signals and globals they reference.
///
/// Idents referenced from an action body that match a state-signal or global
/// name (and are not shadowed by an action parameter) are cloned into the
/// closure before it is moved into place. Zero-parameter actions become
/// `move ||` closures; the others keep their declared typed parameters.
#[must_use]
pub fn action_closure_tokens(ast: &ComponentAst) -> Vec<TokenStream> {
    let state_global_names: std::collections::HashSet<String> = ast
        .state
        .iter()
        .map(|s| s.name.to_string())
        .chain(ast.globals.iter().map(|g| g.name.to_string()))
        .collect();

    ast.actions
        .iter()
        .map(|func| {
            let sig = &func.sig;
            let name = &sig.ident;
            let block = &func.block;

            let param_names: std::collections::HashSet<String> = sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg
                        && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                    {
                        return Some(pat_ident.ident.to_string());
                    }
                    None
                })
                .collect();

            let params: Vec<(proc_macro2::Ident, &syn::Type)> = sig
                .inputs
                .iter()
                .filter_map(|arg| {
                    if let syn::FnArg::Typed(pat_type) = arg
                        && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
                    {
                        return Some((pat_ident.ident.clone(), &*pat_type.ty));
                    }
                    None
                })
                .collect();

            let referenced = collect_block_idents(block);
            let shadows: Vec<_> = referenced
                .iter()
                .filter(|id| {
                    let name_str = id.to_string();
                    state_global_names.contains(&name_str) && !param_names.contains(&name_str)
                })
                .map(|id| {
                    quote! { let #id = #id.clone(); }
                })
                .collect();

            if params.is_empty() {
                quote! {
                    let #name = {
                        #(#shadows)*
                        move || #block
                    };
                }
            } else {
                let param_idents: Vec<_> = params.iter().map(|(id, _)| id).collect();
                let param_types: Vec<_> = params.iter().map(|(_, ty)| ty).collect();
                quote! {
                    let #name = {
                        #(#shadows)*
                        move |#(#param_idents: #param_types),*| #block
                    };
                }
            }
        })
        .collect()
}

/// Whether the user handler takes an event parameter.
///
/// The shared handler calling convention for `keys { .. }` and decorator
/// lowerings: a zero-parameter closure is called with no payload, a
/// one-parameter closure receives the shared `quoin::ev` payload, and any
/// other expression is inlined verbatim (GPUI parity). Defined once here so
/// the Floem and web backends cannot drift.
#[cfg(any(feature = "floem", feature = "leptos", feature = "dioxus"))]
pub(crate) fn handler_arity_is_zero(handler: &Expr) -> bool {
    match handler {
        Expr::Closure(c) => c.inputs.is_empty(),
        _ => false,
    }
}
