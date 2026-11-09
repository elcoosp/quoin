//! The shared macro-time keystroke parser for `keys { .. }` blocks.
//!
//! One grammar (the GPUI backend's) feeds every backend that lowers `keys`:
//! the Floem emitter maps the parsed keystroke onto
//! `floem::ui_events::keyboard` types for its native `KeyDown` listener, and
//! the web emitters (Leptos, Dioxus) hand the DOM key name plus the exact
//! modifier flags to the shared `quoin::keys::keystroke_matches` runtime.
//! Parsing once, here, is what keeps the grammar identical on every backend.
//!
//! Recognized keystroke syntax:
//! - modifiers: `ctrl`, `shift`, `alt`, `cmd`/`super`/`meta` (platform meta)
//! - final segment: a single character (`d`, `1`), `f1`–`f12`, or a named
//!   key (`escape`, `enter`, `tab`, `backspace`, `delete`, `space`, `up`,
//!   `down`, `left`, `right`, `pageup`, `pagedown`, `home`, `end`).

use proc_macro2::TokenStream;
use quote::quote;

/// Modifier flags of one keystroke, as bits (clippy counts raw `bool`
/// fields, not bits).
#[derive(Default, Clone, Copy)]
pub(crate) struct KeyFlags(u8);

impl KeyFlags {
    pub(crate) const CTRL: u8 = 1 << 0;
    pub(crate) const SHIFT: u8 = 1 << 1;
    pub(crate) const ALT: u8 = 1 << 2;
    /// Platform meta (`cmd`/`super`) — matched as its own flag; folded into
    /// `quoin::ev::Key::ctrl` when building the shared payload.
    pub(crate) const META: u8 = 1 << 3;

    /// Whether the flag bit is set.
    pub(crate) fn has(self, bit: u8) -> bool {
        self.0 & bit != 0
    }
}

/// The modifier segments before the final `-key` segment.
fn parse_modifiers(segments: &[&str]) -> Result<KeyFlags, String> {
    let mut flags = KeyFlags::default();
    for modifier in segments {
        match modifier.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" => flags.0 |= KeyFlags::CTRL,
            "shift" => flags.0 |= KeyFlags::SHIFT,
            "alt" | "option" => flags.0 |= KeyFlags::ALT,
            "cmd" | "super" | "meta" | "platform" => flags.0 |= KeyFlags::META,
            "" => {}
            other => return Err(format!("unknown modifier `{other}`")),
        }
    }
    Ok(flags)
}

/// The macro-time parse of one keystroke string.
pub(crate) struct ParsedKeystroke {
    /// Modifier flags for the exact match and the shared payload.
    pub(crate) flags: KeyFlags,
    /// The W3C-style code name (`"KeyD"`, `"Escape"`), reported through
    /// `quoin::ev::Key::code` and mapped onto native `NamedKey` variants by
    /// the Floem emitter.
    pub(crate) code: String,
    /// The DOM `key` value the web matcher compares (`"d"`, `"Escape"`, `" "`).
    pub(crate) dom_key: String,
}

/// The W3C code names that are named keys on every backend: the code equals
/// the native `NamedKey` variant name (`"Escape"` → `NamedKey::Escape`).
/// Consumed only by the Floem emitter's native `Key` mapping — the web
/// matchers compare DOM `key` values, not native variants.
#[cfg(feature = "floem")]
const NAMED_CODES: [&str; 25] = [
    "Escape",
    "Enter",
    "Tab",
    "Backspace",
    "Delete",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "PageUp",
    "PageDown",
    "Home",
    "End",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
];

/// Whether the parsed W3C code name is a named key (the Floem emitter maps
/// it to the same-named `NamedKey` variant); character keys (including
/// `space`) are not.
#[cfg(feature = "floem")]
pub(crate) fn is_named_code(code: &str) -> bool {
    NAMED_CODES.contains(&code)
}

impl ParsedKeystroke {
    /// Parses GPUI-style keystroke syntax; a malformed string yields a
    /// named error that surfaces as a `compile_error!` at the component.
    pub(crate) fn parse(ks: &str) -> Result<Self, String> {
        let mut segments: Vec<&str> = ks.split('-').collect();
        let key_seg = segments
            .pop()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let flags = parse_modifiers(&segments)?;
        let (code, dom_key) = parse_key(&key_seg)?;

        Ok(Self {
            flags,
            code,
            dom_key,
        })
    }

    /// Builds the shared `quoin::ev::Key` payload for one-parameter
    /// handlers; the platform-meta flag reports as `ctrl` (the shared event
    /// type has a single platform-modifier flag).
    pub(crate) fn quoin_key(&self) -> TokenStream {
        let code = self.code.as_str();
        let ctrl = self.flags.has(KeyFlags::CTRL) || self.flags.has(KeyFlags::META);
        let shift = self.flags.has(KeyFlags::SHIFT);
        let alt = self.flags.has(KeyFlags::ALT);
        quote! {
            ::quoin::ev::Key {
                code: #code,
                ctrl: #ctrl,
                shift: #shift,
                alt: #alt,
            }
        }
    }
}

/// The final segment: the W3C code name plus the DOM `key` value the web
/// matcher compares.
fn parse_key(key_seg: &str) -> Result<(String, String), String> {
    // F-keys: `f1`–`f12` map to `NamedKey::F1` .. `NamedKey::F12`.
    if let Some(fkey) = fkey_index(key_seg) {
        let name = format!("F{fkey}");
        return Ok((name.clone(), name));
    }

    Ok(match key_seg {
        "" => return Err("empty keystroke".to_owned()),
        "escape" => ("Escape".to_owned(), "Escape".to_owned()),
        "enter" | "return" => ("Enter".to_owned(), "Enter".to_owned()),
        "tab" => ("Tab".to_owned(), "Tab".to_owned()),
        "backspace" => ("Backspace".to_owned(), "Backspace".to_owned()),
        "delete" | "del" => ("Delete".to_owned(), "Delete".to_owned()),
        // The DOM `key` value for space is the literal space character;
        // the W3C `code` name is `Space`.
        "space" => ("Space".to_owned(), " ".to_owned()),
        "up" => ("ArrowUp".to_owned(), "ArrowUp".to_owned()),
        "down" => ("ArrowDown".to_owned(), "ArrowDown".to_owned()),
        "left" => ("ArrowLeft".to_owned(), "ArrowLeft".to_owned()),
        "right" => ("ArrowRight".to_owned(), "ArrowRight".to_owned()),
        "pageup" => ("PageUp".to_owned(), "PageUp".to_owned()),
        "pagedown" => ("PageDown".to_owned(), "PageDown".to_owned()),
        "home" => ("Home".to_owned(), "Home".to_owned()),
        "end" => ("End".to_owned(), "End".to_owned()),
        single if single.len() == 1 => {
            let ch = single.chars().next().unwrap_or_default();
            let code = if ch.is_ascii_alphabetic() {
                format!("Key{}", ch.to_ascii_uppercase())
            } else if ch.is_ascii_digit() {
                format!("Digit{ch}")
            } else {
                (*single).to_owned()
            };
            (code, (*single).to_owned())
        }
        other => return Err(format!("unknown key `{other}`")),
    })
}

/// `f1`–`f12` → 1–12, anything else → `None`.
fn fkey_index(seg: &str) -> Option<u8> {
    let rest = seg.strip_prefix('f')?;
    let n: u8 = rest.parse().ok()?;
    (1..=12).contains(&n).then_some(n)
}
