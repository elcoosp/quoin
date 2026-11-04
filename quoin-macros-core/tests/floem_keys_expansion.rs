//! Expansion tests for `keys { .. }` on the Floem backend.
//!
//! The keystroke string is parsed at macro time; the expansion must carry a
//! focusable, focus-grabbing root with one `KeyDown` listener whose arms
//! route through the shared runtime matcher (`quoin::floem_keys`) and the
//! shared `quoin::ev::Key` payload.

#![cfg(feature = "floem")]
use quoin_macros_core::emit::render_floem::component::emit_component;
use quoin_macros_core::parse::ComponentAst;

fn expand_component(source: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let ast: ComponentAst = syn::parse2(source).expect("component AST should parse");
    emit_component(&ast)
}

#[test]
fn keys_expand_to_a_key_down_listener_on_the_root() {
    let expanded = expand_component(quote::quote! {
        pub KeysTest {
            state { draft: String = String::new() }
            keys {
                "escape" => draft.set(String::new())
            }
            render {
                quoin_render! { p() { "type here" } }
            }
        }
    });
    let text = expanded.to_string();

    // The expansion is an item (a function), not an expression.
    syn::parse2::<syn::Item>(expanded).expect("component expansion must parse as an item");

    assert!(
        text.contains("listener :: KeyDown"),
        "keys must attach a KeyDown listener; expansion:\n{text}"
    );
    assert!(
        text.contains("request_focus"),
        "the keys root must grab focus (GPUI track_focus contract):\n{text}"
    );
    assert!(
        text.contains("focusable"),
        "the keys root must be focusable:\n{text}"
    );
    // The keystroke was parsed at macro time into an exact native key.
    assert!(
        text.contains("NamedKey :: Escape"),
        "escape must be parsed into NamedKey::Escape at macro time:\n{text}"
    );
    // Dispatch routes through the shared runtime matcher.
    assert!(
        text.contains("floem_keys :: keystroke_matches"),
        "arms must call the shared matcher:\n{text}"
    );
    // Exact-modifier semantics: the mask must be empty for a bare key.
    assert!(
        text.contains("Modifiers :: empty"),
        "bare keystrokes match an empty modifier mask:\n{text}"
    );
}

#[test]
fn keys_handler_can_take_the_shared_key_payload() {
    let expanded = expand_component(quote::quote! {
        pub KeysPayloadTest {
            state { last: String = String::new() }
            keys {
                "ctrl-k" => |key: quoin::ev::Key| last.set(key.code.to_string())
            }
            render {
                quoin_render! { p() { "payload" } }
            }
        }
    });
    let text = expanded.to_string();
    syn::parse2::<syn::Item>(expanded).expect("expansion must parse as an item");

    assert!(
        text.contains(":: quoin :: ev :: Key"),
        "one-parameter handlers receive the shared Key payload:\n{text}"
    );
    assert!(
        text.contains("\"KeyK\""),
        "the W3C-style code name is derived at macro time:\n{text}"
    );
    assert!(
        text.contains("Modifiers :: CONTROL"),
        "ctrl must be part of the exact modifier mask:\n{text}"
    );
}

#[test]
fn malformed_keystroke_is_a_named_compile_error() {
    let expanded = expand_component(quote::quote! {
        pub KeysBadTest {
            state { n: u32 = 0 }
            keys {
                "ctrl-bogus" => n.set(1)
            }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    });
    let text = expanded.to_string();
    assert!(
        text.contains("compile_error") && text.contains("ctrl-bogus"),
        "a malformed keystroke must fail with the input named:\n{text}"
    );
}

#[test]
fn component_without_keys_stays_undecorated() {
    let expanded = expand_component(quote::quote! {
        pub PlainTest {
            state { n: u32 = 0 }
            render {
                quoin_render! { p() { "plain" } }
            }
        }
    });
    let text = expanded.to_string();
    syn::parse2::<syn::Item>(expanded).expect("expansion must parse as an item");
    assert!(
        !text.contains("KeyDown"),
        "no keys section must not emit a listener:\n{text}"
    );
}
