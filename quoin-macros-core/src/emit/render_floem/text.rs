//! Text leaves for the Floem backend: static literals and dynamic closures.
//!
//! Static literals lower to `label(|| "text")` — the closure returns a
//! `&'static str`, so no allocation happens per frame. Dynamic expressions
//! lower to a scoped block that shadow-clones the captured signals and then
//! builds `label(move || expr)`, re-evaluated by Floem's reactivity:
//!
//! ```ignore
//! { let count = ::core::clone::Clone::clone(&count);
//!   ::floem::views::Label::derived(move || format!("Count: {}", count.get())) }
//! ```

use crate::transpile::collect_handler_idents_excluding_params;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// A static string literal: `label(|| "text")`.
pub(crate) fn emit_static_text(lit: &syn::LitStr, style: Option<TokenStream>) -> TokenStream {
    let label = quote! { ::floem::views::Label::derived(|| #lit) };
    match style {
        Some(chain) => quote! { #label #chain },
        None => label,
    }
}

/// A dynamic text expression: shadow-clones the captured signals in a block,
/// then builds the reactive `label`.
pub(crate) fn emit_dynamic_text(
    expr: &Expr,
    style: Option<TokenStream>,
    ctx: &mut super::EmitterCtx,
) -> TokenStream {
    // Capture analysis: signals referenced anywhere in the expression
    // (including inside macro bodies such as `format!`) are shadow-cloned
    // before the `move` closure, so the closure owns its own handles.
    let idents = collect_handler_idents_excluding_params(expr);
    for ident in &idents {
        ctx.capture(ident);
    }
    let shadows: Vec<TokenStream> = idents
        .iter()
        .map(|id| quote! { let #id = ::core::clone::Clone::clone(&#id); })
        .collect();
    let label = if shadows.is_empty() {
        quote! { ::floem::views::Label::derived(move || #expr) }
    } else {
        quote! { { #(#shadows)* ::floem::views::Label::derived(move || #expr) } }
    };
    match style {
        Some(chain) => quote! { #label #chain },
        None => label,
    }
}
