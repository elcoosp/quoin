//! Tests for the shared web keystroke runtime (`quoin_core::keys`) — the
//! `keys { .. }` matcher behind the Leptos and Dioxus lowerings.
//!
//! Matching mirrors the Floem backend's semantics (exact modifiers,
//! case-insensitive character keys, exact named keys). The window-listener
//! attach is a documented no-op on native targets (SSR/native builds have no
//! DOM window), which these tests pin the same way the adapter
//! `clipboard_write_text` fallback is pinned.

use quoin_core::keys::{KeyEvent, Modifiers, attach_window_keydown, keystroke_matches};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn ev(key: &str, modifiers: Modifiers) -> KeyEvent {
    KeyEvent::new("TestKey", key, modifiers)
}

#[test]
fn bare_character_matches_case_insensitively() {
    let expected = "d";
    assert!(
        keystroke_matches(&ev("d", Modifiers::EMPTY), Modifiers::EMPTY, expected),
        "a bare `d` event matches the bare `d` keystroke"
    );
    assert!(
        keystroke_matches(&ev("D", Modifiers::EMPTY), Modifiers::EMPTY, expected),
        "Shift-produced uppercase characters match case-insensitively (Floem parity)"
    );
    assert!(
        !keystroke_matches(&ev("D", Modifiers::CTRL), Modifiers::EMPTY, expected),
        "an extra modifier must not fire a bare binding (exact semantics)"
    );
}

#[test]
fn modifier_flags_are_matched_exactly() {
    let mask = Modifiers::CTRL.union(Modifiers::SHIFT);
    assert!(
        keystroke_matches(&ev("D", mask), mask, "d"),
        "ctrl-shift-d matches ctrl-shift"
    );
    assert!(
        !keystroke_matches(&ev("D", mask.union(Modifiers::ALT)), mask, "d"),
        "an extra Alt must not fire the ctrl-shift binding"
    );
    assert!(
        !keystroke_matches(&ev("D", Modifiers::CTRL), mask, "d"),
        "missing Shift must not fire the ctrl-shift binding"
    );
}

#[test]
fn named_keys_compare_exactly() {
    assert!(
        keystroke_matches(&ev("Escape", Modifiers::EMPTY), Modifiers::EMPTY, "Escape"),
        "the DOM key name `Escape` matches the parsed keystroke"
    );
    assert!(
        !keystroke_matches(&ev("esc", Modifiers::EMPTY), Modifiers::EMPTY, "Escape"),
        "abbreviated names must not match the full named key"
    );
    assert!(
        !keystroke_matches(&ev("Enter", Modifiers::EMPTY), Modifiers::EMPTY, "Escape"),
        "a different named key must not match"
    );
}

#[test]
fn meta_is_its_own_flag_at_match_time() {
    assert!(
        keystroke_matches(&ev("k", Modifiers::META), Modifiers::META, "k"),
        "a `cmd-k` keystroke matches meta-held events"
    );
    assert!(
        !keystroke_matches(&ev("k", Modifiers::CTRL), Modifiers::META, "k"),
        "Ctrl must not fire a cmd binding (the ctrl fold happens only in the \
         macro-built `quoin::ev::Key` payload, not in matching)"
    );
}

#[test]
fn space_matches_the_literal_key_value() {
    assert!(
        keystroke_matches(&ev(" ", Modifiers::EMPTY), Modifiers::EMPTY, " "),
        "space parses to the DOM key value ` ` and matches it"
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn native_attach_registers_nothing_and_remove_is_a_noop() {
    // The SSR/native platform boundary: no DOM window exists, so the attach
    // must register no listener (the same documented fallback class as the
    // adapters' `clipboard_write_text`), and `remove` must stay callable.
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let handle = attach_window_keydown(move |_ev: &KeyEvent| {
        counter.fetch_add(1, Ordering::Relaxed);
    });
    handle.remove();
    assert_eq!(
        calls.load(Ordering::Relaxed),
        0,
        "a native attach must never invoke the handler (no DOM window)"
    );
}
