use proc_macro::TokenStream;
use quoin_macros_core::emit::FrameworkEmitter;

fn emitter() -> quoin_macros_core::emit::Emitter {
    quoin_macros_core::emit::Emitter
}

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
    let ast = match syn::parse::<quoin_macros_core::parse::ComponentAst>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };
    emitter().emit_component(&ast).into()
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
    let ast = match syn::parse::<quoin_macros_core::render_ast::RenderNode>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };
    emitter().emit_render(&ast).into()
}

#[proc_macro]
pub fn quoin_element(input: TokenStream) -> TokenStream {
    let def = match syn::parse::<quoin_macros_core::custom_element::CustomElementDef>(input) {
        Ok(def) => def,
        Err(e) => return e.to_compile_error().into(),
    };
    quoin_macros_core::custom_element::expand_custom_element(def).into()
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
    emitter().emit_effect(&eff).into()
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
    let ast = match syn::parse::<quoin_macros_core::run_app::RunAppInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };
    emitter().emit_run_app(&ast).into()
}
