// Imports in this module vary with the `leptos-shadcn`/`dioxus-shadcn` feature:
// some are only used when the shadcn emit path is active. A file-level allow
// keeps every feature combination compiling without per-line noise.

use proc_macro2::TokenStream;
#[cfg(feature = "leptos-shadcn")]
use quote::quote;
use std::sync::atomic::{AtomicUsize, Ordering};

// Only exercised by the `leptos-shadcn` code path; kept unconditionally so
// every feature combination compiles.

static EXTRACT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn next_extract_id() -> usize {
    EXTRACT_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Returns the element identifier to use in view!.
/// - shadcn ON:  imports the shadcn component as a local alias, returns the alias ident
/// - shadcn OFF: returns the plain HTML tag ident
#[cfg(feature = "leptos-shadcn")]
pub(crate) fn import_shadcn_or_html_tag(
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
pub(crate) fn import_shadcn_or_html_tag(
    _bindings: &mut Vec<TokenStream>,
    _shadcn_comp: &str,
    html_tag: &str,
) -> proc_macro2::Ident {
    quote::format_ident!("{}", html_tag)
}
