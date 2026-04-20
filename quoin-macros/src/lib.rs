use proc_macro::TokenStream;
use quote::quote;

mod parse;
mod emit;
mod render_ast;
mod transpile;
mod custom_element;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    if input_str.contains("DEBUG") {
        return TokenStream::from(quote! {
            compile_error!(#input_str);
        });
    }

    let ast = match syn::parse::<parse::ComponentAst>(input) {
        Ok(ast) => ast,
        Err(e) => {
            let err = e.to_compile_error();
            return TokenStream::from(quote! { #err });
        }
    };

    #[cfg(feature = "gpui")]
    let tokens = emit::gpui::emit_component(&ast);

    #[cfg(feature = "leptos")]
    let tokens = emit::leptos::emit_component(&ast);

    #[cfg(feature = "dioxus")]
    let tokens = emit::dioxus::emit_component(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote! { compile_error!("component! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    tokens.into()
}

#[proc_macro]
pub fn quoin_render(input: TokenStream) -> TokenStream {
    let ast = match syn::parse::<render_ast::RenderNode>(input) {
        Ok(ast) => ast,
        Err(e) => {
            let err = e.to_compile_error();
            return TokenStream::from(quote! { #err });
        }
    };

    #[cfg(feature = "gpui")]
    let tokens = emit::render_gpui::emit_render(&ast);

    #[cfg(feature = "leptos")]
    let tokens = emit::render_leptos::emit_render(&ast);

    #[cfg(feature = "dioxus")]
    let tokens = emit::render_dioxus::emit_render(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote! { compile_error!("quoin_render! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    tokens.into()
}

#[proc_macro]
pub fn quoin_element(input: TokenStream) -> TokenStream {
    let def = match syn::parse::<custom_element::CustomElementDef>(input) {
        Ok(def) => def,
        Err(e) => {
            let err = e.to_compile_error();
            return TokenStream::from(quote! { #err });
        }
    };
    custom_element::expand_custom_element(def).into()
}
