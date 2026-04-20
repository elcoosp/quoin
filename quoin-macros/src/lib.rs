use proc_macro::TokenStream;

#[proc_macro]
pub fn component(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro]
pub fn quoin_render(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
