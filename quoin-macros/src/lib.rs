use proc_macro::TokenStream;
use quote::quote;

mod custom_element;
mod emit;
mod parse;
mod render_ast;
mod transpile;

#[manyhow::manyhow]
#[proc_macro]
pub fn component(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let ast = syn::parse::<parse::ComponentAst>(input)?;

    #[cfg(feature = "gpui")]
    let tokens = emit::gpui::emit_component(&ast);

    #[cfg(feature = "leptos")]
    let tokens = emit::leptos::emit_component(&ast);

    #[cfg(feature = "dioxus")]
    let tokens = emit::dioxus::emit_component(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote! { compile_error!("component! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    Ok(tokens.into())
}

#[manyhow::manyhow]
#[proc_macro]
pub fn quoin_render(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let ast = syn::parse::<render_ast::RenderNode>(input)?;

    #[cfg(feature = "gpui")]
    let tokens = emit::render_gpui::emit_render(&ast);

    #[cfg(feature = "leptos")]
    let tokens = emit::render_leptos::emit_render(&ast);

    #[cfg(feature = "dioxus")]
    let tokens = emit::render_dioxus::emit_render(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote! { compile_error!("quoin_render! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    Ok(tokens.into())
}

#[manyhow::manyhow]
#[proc_macro]
pub fn quoin_element(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let def = syn::parse::<custom_element::CustomElementDef>(input)?;
    Ok(custom_element::expand_custom_element(def).into())
}
