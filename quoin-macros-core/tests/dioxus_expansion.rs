//! Expansion-shape tests for the Dioxus component emitter (the expansion
//! gate), mirroring `xilem_expansion.rs`. The emitter produces token streams
//! only — the framework itself is never linked here.

#![cfg(feature = "dioxus")]
use quoin_macros_core::emit::dioxus::emit_component;
use quoin_macros_core::parse::ComponentAst;

fn expand(source: proc_macro2::TokenStream) -> String {
    let ast: ComponentAst = syn::parse2(source).expect("component AST should parse");
    emit_component(&ast).to_string()
}

#[test]
fn component_expands_to_a_dioxus_component_with_hooks() {
    let text = expand(quote::quote! {
        pub Counter {
            state { n: u32 = 0 }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    assert!(
        text.contains("dioxus :: prelude :: component"),
        "expected the #[component] attribute:\n{text}"
    );
    assert!(
        text.contains("use_hook"),
        "state must initialize inside a use_hook:\n{text}"
    );
}

#[test]
fn component_render_ending_in_a_let_gets_the_element_fallback() {
    // Dioxus 0.7 component bodies must return `Element`; a render block whose
    // last statement is a `let` binding would return `()`, so the emitter
    // appends the `VNode::placeholder()` fallback.
    let text = expand(quote::quote! {
        pub WithLet {
            state { n: u32 = 0 }
            render {
                let _marker = 1;
            }
        }
    });
    assert!(
        text.contains("VNode :: placeholder"),
        "a block-final let binding needs the Element fallback:\n{text}"
    );
}

#[test]
fn component_without_keys_expands_cleanly() {
    // The keys diagnostic must never fire on a component that does not use
    // `keys { .. }` — no false positives.
    let text = expand(quote::quote! {
        pub Plain {
            state { n: u32 = 0 }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    assert!(
        !text.contains("compile_error"),
        "a keys-free component must expand without a diagnostic:\n{text}"
    );
}

#[test]
fn component_with_keys_is_a_named_diagnostic() {
    let text = expand(quote::quote! {
        pub KeysApp {
            state { n: u32 = 0 }
            keys { "escape" => n.set(0) }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    assert!(
        text.contains("compile_error") && text.contains("keys"),
        "keys on Dioxus must fail with the documented diagnostic, not silently \
         drop the block:\n{text}"
    );
    assert!(
        text.contains("Dioxus"),
        "the diagnostic must name the backend:\n{text}"
    );
    assert!(
        text.contains("docs/BACKEND-PARITY.md"),
        "the diagnostic must point at the contract document:\n{text}"
    );
}

#[test]
fn component_unmount_hooks_lower_to_use_drop() {
    let text = expand(quote::quote! {
        pub Hooked {
            state { n: u32 = 0 }
            on_unmount { log::info!("bye"); }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    assert!(
        text.contains("use_drop"),
        "on_unmount must lower to dioxus::prelude::use_drop:\n{text}"
    );
}
