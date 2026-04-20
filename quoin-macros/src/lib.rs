use proc_macro::TokenStream;
use syn::parse_macro_input;

mod parse;
mod emit;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as parse::ComponentAst);
    let tokens = emit::gpui::emit_component(&ast);
    tokens.into()
}

#[proc_macro]
pub fn quoin_render(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
