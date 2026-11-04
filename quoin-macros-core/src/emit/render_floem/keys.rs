//! `keys { "keystroke" => handler }` for the Floem backend.
//!
//! The keystroke string is parsed **at macro time** into an exact modifier
//! mask plus an expected native `Key` value; each entry becomes one `if` arm
//! inside a single `.on_event(listener::KeyDown, …)` decorator on the
//! component root (see `component.rs`). The runtime matcher lives in
//! `quoin::floem_keys`, so the per-keystroke cost is one modifiers
//! comparison plus one key comparison — the same cost a hand-written Floem
//! `KeyDown` handler pays.
//!
//! Recognized keystroke syntax (the GPUI backend's grammar):
//! - modifiers: `ctrl`, `shift`, `alt`, `cmd`/`super`/`meta` (platform meta)
//! - final segment: a single character (`d`, `1`), `f1`–`f12`, or a named
//!   key (`escape`, `enter`, `tab`, `backspace`, `delete`, `space`, `up`,
//!   `down`, `left`, `right`, `pageup`, `pagedown`, `home`, `end`).

use crate::emit::render_floem::handler;
use crate::transpile::collect_handler_idents_excluding_params;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// One compiled `keys` entry.
pub(crate) struct KeyArm {
    /// Shadow clones the handler needs (component state/globals it
    /// references) — emitted once per component, ahead of the root view.
    pub shadows: Vec<TokenStream>,
    /// The `if` arm (or a `compile_error!` for a malformed keystroke),
    /// evaluating to an `EventPropagation` value.
    pub arm: TokenStream,
}

/// Lowers one `keys` entry.
///
/// `state_globals` holds the component's state-signal and global names so
/// only those references are shadow-cloned (cloning a plain function name
/// would fail to compile).
#[must_use]
pub(crate) fn lower_key_entry(
    keystroke: &str,
    handler: &Expr,
    state_globals: &std::collections::HashSet<String>,
) -> KeyArm {
    let parsed = match ParsedKeystroke::parse(keystroke) {
        Ok(parsed) => parsed,
        Err(why) => {
            let message = format!("invalid `keys` keystroke `{keystroke}`: {why}");
            return KeyArm {
                shadows: Vec::new(),
                arm: quote! { ::core::compile_error!(#message) },
            };
        }
    };

    // Handler calling convention (shared with the other decorators):
    // a zero-arg closure is called, a one-arg closure receives the shared
    // `quoin::ev::Key`, any other expression is inlined (GPUI parity).
    let call = if handler::handler_arity_is_zero(handler) {
        quote! { (#handler)(); }
    } else if matches!(handler, Expr::Closure(_)) {
        let ev_key = parsed.quoin_key();
        quote! { (#handler)(#ev_key); }
    } else {
        quote! { #handler; }
    };

    let shadows: Vec<TokenStream> = collect_handler_idents_excluding_params(handler)
        .into_iter()
        .filter(|id| state_globals.contains(&id.to_string()))
        .map(|id| quote! { let #id = ::core::clone::Clone::clone(&#id); })
        .collect();

    let modifiers = parsed.modifiers;
    let key_expr = parsed.key_expr;
    let arm = quote! {
        if ::quoin::floem_keys::keystroke_matches(__ev, #modifiers, &#key_expr) {
            #call
            ::floem::event::EventPropagation::Stop
        }
    };
    KeyArm { shadows, arm }
}

/// Modifier flags of one keystroke, as bits (clippy counts raw `bool`
/// fields, not bits).
#[derive(Default, Clone, Copy)]
struct KeyFlags(u8);

impl KeyFlags {
    const CTRL: u8 = 1 << 0;
    const SHIFT: u8 = 1 << 1;
    const ALT: u8 = 1 << 2;
    /// Platform meta (`cmd`/`super`) — matched via `Modifiers::META` and
    /// reported as `ctrl` on the shared payload.
    const META: u8 = 1 << 3;

    fn has(self, bit: u8) -> bool {
        self.0 & bit != 0
    }
}

/// The macro-time parse of one keystroke string.
struct ParsedKeystroke {
    /// `Modifiers::… | Modifiers::…` — an exact mask.
    modifiers: TokenStream,
    /// The expected native `Key` value expression.
    key_expr: TokenStream,
    /// The W3C-style code name reported through `quoin::ev::Key::code`.
    code: String,
    /// Modifier flags for the mask and the shared payload.
    flags: KeyFlags,
}

impl ParsedKeystroke {
    /// Parses GPUI-style keystroke syntax; a malformed string yields a
    /// named error that surfaces as a `compile_error!` at the component.
    fn parse(ks: &str) -> Result<Self, String> {
        let mut segments: Vec<&str> = ks.split('-').collect();
        let key_seg = segments
            .pop()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let flags = Self::parse_modifiers(segments)?;
        let (key_expr, code) = Self::parse_key(&key_seg)?;

        let modifiers = flags.modifier_mask();
        Ok(Self {
            modifiers,
            key_expr,
            code,
            flags,
        })
    }

    /// The modifier segments before the final `-key` segment.
    fn parse_modifiers(segments: Vec<&str>) -> Result<KeyFlags, String> {
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

    /// The final segment: the native `Key` expression plus the W3C-style
    /// code name.
    fn parse_key(key_seg: &str) -> Result<(TokenStream, String), String> {
        let kb = quote! { ::floem::ui_events::keyboard };

        // F-keys: `f1`–`f12` map to `NamedKey::F1` .. `NamedKey::F12`.
        if let Some(fkey) = fkey_index(key_seg) {
            let variant = quote::format_ident!("F{fkey}");
            return Ok((
                quote! { #kb::Key::Named(#kb::NamedKey::#variant) },
                format!("F{fkey}"),
            ));
        }

        Ok(match key_seg {
            "" => return Err("empty keystroke".to_owned()),
            "escape" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Escape) },
                "Escape".to_owned(),
            ),
            "enter" | "return" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Enter) },
                "Enter".to_owned(),
            ),
            "tab" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Tab) },
                "Tab".to_owned(),
            ),
            "backspace" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Backspace) },
                "Backspace".to_owned(),
            ),
            "delete" | "del" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Delete) },
                "Delete".to_owned(),
            ),
            "space" => (
                quote! { #kb::Key::Character(" ".to_string()) },
                "Space".to_owned(),
            ),
            "up" => (
                quote! { #kb::Key::Named(#kb::NamedKey::ArrowUp) },
                "ArrowUp".to_owned(),
            ),
            "down" => (
                quote! { #kb::Key::Named(#kb::NamedKey::ArrowDown) },
                "ArrowDown".to_owned(),
            ),
            "left" => (
                quote! { #kb::Key::Named(#kb::NamedKey::ArrowLeft) },
                "ArrowLeft".to_owned(),
            ),
            "right" => (
                quote! { #kb::Key::Named(#kb::NamedKey::ArrowRight) },
                "ArrowRight".to_owned(),
            ),
            "pageup" => (
                quote! { #kb::Key::Named(#kb::NamedKey::PageUp) },
                "PageUp".to_owned(),
            ),
            "pagedown" => (
                quote! { #kb::Key::Named(#kb::NamedKey::PageDown) },
                "PageDown".to_owned(),
            ),
            "home" => (
                quote! { #kb::Key::Named(#kb::NamedKey::Home) },
                "Home".to_owned(),
            ),
            "end" => (
                quote! { #kb::Key::Named(#kb::NamedKey::End) },
                "End".to_owned(),
            ),
            single if single.len() == 1 => {
                let ch = single.chars().next().unwrap_or_default();
                let code = if ch.is_ascii_alphabetic() {
                    format!("Key{}", ch.to_ascii_uppercase())
                } else if ch.is_ascii_digit() {
                    format!("Digit{ch}")
                } else {
                    (*single).to_owned()
                };
                (quote! { #kb::Key::Character(#single.to_string()) }, code)
            }
            other => return Err(format!("unknown key `{other}`")),
        })
    }

    /// Builds the shared `quoin::ev::Key` payload for one-parameter
    /// handlers; the platform-meta flag reports as `ctrl` (the shared event
    /// type has a single platform-modifier flag).
    fn quoin_key(&self) -> TokenStream {
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

/// The exact-modifier mask as native `Modifiers` tokens.
impl KeyFlags {
    fn modifier_mask(self) -> TokenStream {
        let kb = quote! { ::floem::ui_events::keyboard };
        let mut mask: Option<TokenStream> = None;
        for (held, flag) in [
            (self.has(KeyFlags::CTRL), quote! { #kb::Modifiers::CONTROL }),
            (self.has(KeyFlags::SHIFT), quote! { #kb::Modifiers::SHIFT }),
            (self.has(KeyFlags::ALT), quote! { #kb::Modifiers::ALT }),
            (self.has(KeyFlags::META), quote! { #kb::Modifiers::META }),
        ] {
            if held {
                mask = Some(match mask {
                    Some(prev) => quote! { #prev | #flag },
                    None => flag,
                });
            }
        }
        mask.unwrap_or_else(|| quote! { #kb::Modifiers::empty() })
    }
}

/// `f1`–`f12` → 1–12, anything else → `None`.
fn fkey_index(seg: &str) -> Option<u8> {
    let rest = seg.strip_prefix('f')?;
    let n: u8 = rest.parse().ok()?;
    (1..=12).contains(&n).then_some(n)
}
