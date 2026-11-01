use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Wraps `inner` in the `#[cfg(..)]` attributes found on the element, so
/// conditional compilation flows through the generated code.
pub fn wrap_with_cfg(attrs: &[syn::Attribute], inner: TokenStream) -> TokenStream {
    let cfg_attrs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    if cfg_attrs.is_empty() {
        inner
    } else {
        quote! { { #(#cfg_attrs)* { #inner } } }
    }
}
