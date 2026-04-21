use proc_macro::TokenStream;
use syn::parse_macro_input;

mod custom_element;
mod effect;
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
    let tokens = quote::quote! { compile_error!("component! requires a framework feature"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens =
        quote::quote! { compile_error!("Only one framework feature may be enabled at a time."); };

    tokens.into()
}

// FIX: Parse raw tokens directly to bypass Rust's strict expression parser
#[proc_macro]
pub fn quoin_render(input: TokenStream) -> TokenStream {
    let ast = match syn::parse::<render_ast::RenderNode>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = emit::render_gpui::emit_render(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = emit::render_leptos::emit_render(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = emit::render_dioxus::emit_render(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("quoin_render! requires a framework feature"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens =
        quote::quote! { compile_error!("Only one framework feature may be enabled at a time."); };

    tokens.into()
}

#[proc_macro]
pub fn quoin_element(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as custom_element::CustomElementDef);
    custom_element::expand_custom_element(def).into()
}

#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    let eff = parse_macro_input!(input as effect::Effect);
    let body = &eff.body;

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = {
        quote::quote! {{ (#body)(); }}
    };

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = {
        quote::quote! { leptos::prelude::create_effect(move |_| { #body; }); }
    };

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = {
        quote::quote! { dioxus::prelude::use_effect(move || { #body; }); }
    };

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("effect! requires a framework feature"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens =
        quote::quote! { compile_error!("Only one framework feature may be enabled at a time."); };

    tokens.into()
}
