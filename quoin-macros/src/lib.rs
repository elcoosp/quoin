// quoin-macros/src/lib.rs
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod custom_element;
mod emit;
mod parse;
mod render_ast;
mod transpile;

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as parse::ComponentAst);

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = emit::gpui::emit_component(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = emit::leptos::emit_component(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = emit::dioxus::emit_component(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("component! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens = quote::quote! { compile_error!("Only one framework feature may be enabled at a time for quoin-macros."); };

    tokens.into()
}

#[proc_macro]
pub fn quoin_render(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as render_ast::RenderNode);

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = emit::render_gpui::emit_render(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = emit::render_leptos::emit_render(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = emit::render_dioxus::emit_render(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("quoin_render! requires a framework feature (e.g., 'gpui', 'leptos', 'dioxus')"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens = quote::quote! { compile_error!("Only one framework feature may be enabled at a time for quoin-macros."); };

    tokens.into()
}

#[proc_macro]
pub fn quoin_element(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as custom_element::CustomElementDef);
    custom_element::expand_custom_element(def).into()
}
