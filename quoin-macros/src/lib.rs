use proc_macro::TokenStream;

#[proc_macro]
/// Defines a framework-agnostic reactive component.
///
/// # Syntax
///
/// ```ignore
/// component! {
///     pub MyComponent {
///         props { title: String = "".into() }
///         state { count: u32 = 0 }
///         render { ... }
///     }
/// }
/// ```
///
/// # Framework output
/// - **GPUI**: Generates a struct with a `Render` impl.
/// - **Leptos**: Generates a `#[component]` function.
/// - **Dioxus**: Generates a `#[component]` function.
pub fn component(input: TokenStream) -> TokenStream {
    #[allow(unused)]
    let ast = match syn::parse::<quoin_macros_core::parse::ComponentAst>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::gpui::emit_component(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::leptos::emit_component(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = quoin_macros_core::emit::dioxus::emit_component(&ast);

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

#[proc_macro]
/// Declarative UI macro with Tailwind-like syntax.
///
/// Supports elements (`div`, `button`, etc.), special components
/// (`tabs`, `data_table`, ...), `if`/`for` control flow, and expressions.
///
/// # Example
///
/// ```ignore
/// quoin_render! {
///     div(class: "flex p-4") {
///         "Hello"
///         if[show] { "World" }
///     }
/// }
/// ```
pub fn quoin_render(input: TokenStream) -> TokenStream {
    #[allow(unused)]
    let ast = match syn::parse::<quoin_macros_core::render_ast::RenderNode>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::render_gpui::emit_render(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::render_leptos::emit_render(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = quoin_macros_core::emit::render_dioxus::emit_render(&ast);

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
    #[allow(unused)]
    let def = match syn::parse::<quoin_macros_core::custom_element::CustomElementDef>(input) {
        Ok(def) => def,
        Err(e) => return e.to_compile_error().into(),
    };

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("quoin_element! requires a framework feature"); };

    #[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus"))]
    let tokens = quoin_macros_core::custom_element::expand_custom_element(def);

    tokens.into()
}

#[proc_macro]
/// Reactive side effect with optional cleanup.
///
/// # Syntax
///
/// ```ignore
/// effect! { deps: [count], run: || println!("{}", count.get()) }
/// effect! { deps: [query], run: || fetch(), cleanup: || cancel() }
/// ```
pub fn effect(input: TokenStream) -> TokenStream {
    let eff = match syn::parse::<quoin_macros_core::effect::Effect>(input) {
        Ok(eff) => eff,
        Err(e) => return e.to_compile_error().into(),
    };

    #[allow(unused)]
    let body = &eff.body;
    #[allow(unused)]
    let cleanup = &eff.cleanup;

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = match cleanup {
        Some(_cleanup_expr) => quote::quote! {
            compile_error!(
                "effect! cleanup is not supported in GPUI.\n\n\
                 GPUI has no effect scoping mechanism. If you need cleanup \
                 logic, handle it manually in your view's Drop impl or use \
                 a dedicated cleanup function called on unmount."
            )
        },
        None => quote::quote! {{
            (#body)();
        }},
    };

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = match cleanup {
        Some(cleanup_expr) => quote::quote! {
            leptos::prelude::create_effect(move |_| {
                #body;
            });
            leptos::prelude::on_cleanup(move || {
                #cleanup_expr;
            });
        },
        None => quote::quote! {
            leptos::prelude::create_effect(move |_| {
                #body;
            });
        },
    };

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = match cleanup {
        Some(cleanup_expr) => quote::quote! {
            dioxus::prelude::use_effect(move || {
                #body;
            });
            dioxus::prelude::use_drop(move || {
                #cleanup_expr;
            });
        },
        None => quote::quote! {
            dioxus::prelude::use_effect(move || {
                #body;
            });
        },
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

#[proc_macro]
/// Bootstrap an application with a single line.
///
/// # Example
///
/// ```ignore
/// run_app!(MyComponent);
/// run_app!(MyComponent, window_opts: custom_window_options);
/// ```
pub fn run_app(input: TokenStream) -> TokenStream {
    #[allow(unused)]
    let ast = match syn::parse::<quoin_macros_core::run_app::RunAppInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    #[cfg(all(feature = "gpui", not(any(feature = "leptos", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::run_app_gpui::emit_run_app(&ast);

    #[cfg(all(feature = "leptos", not(any(feature = "gpui", feature = "dioxus"))))]
    let tokens = quoin_macros_core::emit::run_app_leptos::emit_run_app(&ast);

    #[cfg(all(feature = "dioxus", not(any(feature = "gpui", feature = "leptos"))))]
    let tokens = quoin_macros_core::emit::run_app_dioxus::emit_run_app(&ast);

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let tokens = quote::quote! { compile_error!("run_app! requires a framework feature"); };

    #[cfg(any(
        all(feature = "gpui", feature = "leptos"),
        all(feature = "gpui", feature = "dioxus"),
        all(feature = "leptos", feature = "dioxus"),
    ))]
    let tokens =
        quote::quote! { compile_error!("Only one framework feature may be enabled at a time."); };

    tokens.into()
}
