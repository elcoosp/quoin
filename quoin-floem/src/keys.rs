//! Keystroke matching for the `keys { .. }` lowering.
//!
//! The macro emitter parses GPUI-style keystroke strings (`"ctrl-shift-d"`,
//! `"escape"`) at compile time and generates a call to
//! [`keystroke_matches`] per entry, so the runtime cost of a keystroke check
//! is one modifiers comparison plus one key comparison — the same cost a
//! hand-written Floem `KeyDown` handler pays.
//!
//! Semantics mirror the GPUI backend's `KeyBinding` matching:
//! - modifiers are matched **exactly** (`"ctrl-shift-d"` does not fire when
//!   Alt is also held),
//! - character keys match case-insensitively (Shift produces uppercase
//!   characters on real keyboards),
//! - `cmd`/`super`/`meta` map to the platform meta modifier and to
//!   `quoin::ev::Key::ctrl` (the shared event type has a single platform
//!   modifier flag).

use floem::ui_events::keyboard::{Key, KeyState, KeyboardEvent, Modifiers};

/// Whether a `KeyDown` event matches a parsed keystroke: exact modifiers
/// plus a case-insensitive key comparison.
#[must_use]
#[inline]
pub fn keystroke_matches(ev: &KeyboardEvent, modifiers: Modifiers, expected: &Key) -> bool {
    ev.state == KeyState::Down && ev.modifiers == modifiers && keys_equal(&ev.key, expected)
}

/// Character keys compare case-insensitively (Shift yields uppercase
/// characters on real keyboards); named keys compare by value.
#[must_use]
fn keys_equal(actual: &Key, expected: &Key) -> bool {
    match (actual, expected) {
        (Key::Character(a), Key::Character(b)) => a.eq_ignore_ascii_case(b),
        _ => actual == expected,
    }
}

/// Unit tests for the matching semantics the macro output relies on.
#[cfg(test)]
mod tests {
    use super::*;

    fn event(key: Key, modifiers: Modifiers) -> KeyboardEvent {
        KeyboardEvent {
            state: KeyState::Down,
            key,
            code: Code::Unidentified,
            location: Location::Standard,
            modifiers,
            repeat: false,
            is_composing: false,
        }
    }

    use floem::ui_events::keyboard::{Code, Location, NamedKey};

    #[test]
    fn character_match_is_case_insensitive() {
        let ev = event(Key::Character("D".to_string()), Modifiers::empty());
        let d = Key::Character("d".to_string());
        assert!(keystroke_matches(&ev, Modifiers::empty(), &d));
        // The same key with a modifier held must not fire a bare binding.
        assert!(!keystroke_matches(&ev, Modifiers::CONTROL, &d));
    }

    #[test]
    fn named_and_modifier_matches_are_exact() {
        let ev = event(
            Key::Named(NamedKey::Escape),
            Modifiers::CONTROL | Modifiers::SHIFT,
        );
        let esc = Key::Named(NamedKey::Escape);
        assert!(keystroke_matches(
            &ev,
            Modifiers::CONTROL | Modifiers::SHIFT,
            &esc
        ));
        // Extra modifier held: no match (exact semantics).
        assert!(!keystroke_matches(
            &ev,
            Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT,
            &esc
        ));
        assert!(!keystroke_matches(&ev, Modifiers::CONTROL, &esc));
    }

    #[test]
    fn key_up_does_not_match() {
        let mut ev = event(Key::Named(NamedKey::Enter), Modifiers::empty());
        ev.state = KeyState::Up;
        assert!(!keystroke_matches(
            &ev,
            Modifiers::empty(),
            &Key::Named(NamedKey::Enter)
        ));
    }
}
