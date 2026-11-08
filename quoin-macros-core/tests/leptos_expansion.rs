//! Expansion-shape tests for the Leptos component emitter (the expansion
//! gate), mirroring `xilem_expansion.rs`. The emitter produces token streams
//! only — the framework itself is never linked here.

#![cfg(feature = "leptos")]
use quoin_macros_core::emit::leptos::emit_component;
use quoin_macros_core::parse::ComponentAst;

fn expand(source: proc_macro2::TokenStream) -> String {
    let ast: ComponentAst = syn::parse2(source).expect("component AST should parse");
    emit_component(&ast).to_string()
}

#[test]
fn component_expands_to_a_leptos_component_with_signals() {
    let text = expand(quote::quote! {
        pub Counter {
            state { n: u32 = 0 }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    assert!(
        text.contains("leptos :: prelude :: component"),
        "expected the #[component] attribute:\n{text}"
    );
    assert!(
        text.contains("ctx . create_signal (0)"),
        "state must initialize through the adapter context:\n{text}"
    );
    assert!(
        text.contains("move ||"),
        "the render closure must capture the shadowed signals:\n{text}"
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
        "keys on Leptos must fail with the documented diagnostic, not silently \
         drop the block:\n{text}"
    );
    assert!(
        text.contains("Leptos"),
        "the diagnostic must name the backend:\n{text}"
    );
    assert!(
        text.contains("docs/BACKEND-PARITY.md"),
        "the diagnostic must point at the contract document:\n{text}"
    );
}

#[test]
fn component_unmount_hooks_lower_to_on_cleanup() {
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
        text.contains("on_cleanup"),
        "on_unmount must lower to leptos::prelude::on_cleanup:\n{text}"
    );
}
