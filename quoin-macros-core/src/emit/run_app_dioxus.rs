use crate::run_app::RunAppInput;
use proc_macro2::TokenStream;
use quote::quote;

#[must_use]
/// Expands `run_app!` into the Dioxus application bootstrap (`main`).
pub fn emit_run_app(input: &RunAppInput) -> TokenStream {
    let component = &input.component;
    quote! {
        fn main() {
            dioxus::launch(#component);
        }
    }
}
