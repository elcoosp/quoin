use proc_macro::TokenStream;
use syn::parse_macro_input;
use quote::quote;

mod parse;
mod emit;
mod render_ast;
mod transpile;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as parse::ComponentAst);
    let tokens = emit::gpui::emit_component(&ast);
    tokens.into()
}

#[proc_macro]
pub fn quoin_render(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as render_ast::RenderNode);

    #[cfg(feature = "gpui")]
    let tokens = emit::render_gpui::emit_render(&ast);

    #[cfg(not(feature = "gpui"))]
    let tokens = quote! { compile_error!("quoin_render! requires a framework feature (e.g., 'gpui')"); };

    tokens.into()
}
