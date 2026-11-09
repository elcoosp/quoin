//! `keys { "keystroke" => handler }` for the Floem backend.
//!
//! The keystroke string is parsed **at macro time** by the shared parser
//! ([`crate::emit::keystroke`]) into an exact modifier mask plus an expected
//! native `Key` value; each entry becomes one `if` arm inside a single
//! `.on_event(listener::KeyDown, …)` decorator on the component root (see
//! `component.rs`). The runtime matcher lives in `quoin::floem_keys`, so the
//! per-keystroke cost is one modifiers comparison plus one key comparison —
//! the same cost a hand-written Floem `KeyDown` handler pays.
//!
//! Semantics mirror the GPUI backend's `KeyBinding` matching:
//! - modifiers are matched **exactly** (`"ctrl-shift-d"` does not fire when
//!   Alt is also held),
//! - character keys match case-insensitively (Shift produces uppercase
//!   characters on real keyboards),
//! - `cmd`/`super`/`meta` map to the platform meta modifier and to
//!   `quoin::ev::Key::ctrl` (the shared event type has a single platform
//!   modifier flag).

use crate::emit::common::handler_arity_is_zero;
use crate::emit::keystroke::{KeyFlags, ParsedKeystroke, is_named_code};
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
    let call = if handler_arity_is_zero(handler) {
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

    let modifiers = modifier_mask(parsed.flags);
    let key_expr = floem_key_expr(&parsed);
    let arm = quote! {
        if ::quoin::floem_keys::keystroke_matches(__ev, #modifiers, &#key_expr) {
            #call
            ::floem::event::EventPropagation::Stop
        }
    };
    KeyArm { shadows, arm }
}

/// The exact-modifier mask as native `Modifiers` tokens.
fn modifier_mask(flags: KeyFlags) -> TokenStream {
    let kb = quote! { ::floem::ui_events::keyboard };
    let mut mask: Option<TokenStream> = None;
    for (held, flag) in [
        (
            flags.has(KeyFlags::CTRL),
            quote! { #kb::Modifiers::CONTROL },
        ),
        (flags.has(KeyFlags::SHIFT), quote! { #kb::Modifiers::SHIFT }),
        (flags.has(KeyFlags::ALT), quote! { #kb::Modifiers::ALT }),
        (flags.has(KeyFlags::META), quote! { #kb::Modifiers::META }),
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

/// The expected native `Key` value: named keys map to the same-named
/// `NamedKey` variant (the W3C code name is the variant name), character
/// keys (including `space`) to `Key::Character`.
fn floem_key_expr(parsed: &ParsedKeystroke) -> TokenStream {
    let kb = quote! { ::floem::ui_events::keyboard };
    if is_named_code(&parsed.code) {
        let variant = quote::format_ident!("{}", parsed.code);
        quote! { #kb::Key::Named(#kb::NamedKey::#variant) }
    } else {
        let ch = parsed.dom_key.as_str();
        quote! { #kb::Key::Character(#ch.to_string()) }
    }
}
