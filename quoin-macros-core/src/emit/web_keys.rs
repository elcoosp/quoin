//! `keys { "keystroke" => handler }` for the web backends (Leptos, Dioxus).
//!
//! The keystroke string is parsed **at macro time** by the shared parser
//! ([`crate::emit::keystroke`]) into the exact modifier flags plus the DOM
//! key name; each entry becomes one `if` arm inside a **single**
//! window-level `keydown` listener per component, registered through
//! `quoin::keys::attach_window_keydown` and disposed with the component's
//! cleanup hook — the web counterpart of the Floem one-listener contract.
//! The per-keystroke cost is four flag comparisons plus one key comparison
//! (see `quoin::keys`).
//!
//! Handler state the arms reference (component signals and globals) is
//! shadow-cloned into the listener closure, exactly like the Floem
//! lowering, so the component's own bindings stay live.

use crate::emit::common::handler_arity_is_zero;
use crate::emit::keystroke::{KeyFlags, ParsedKeystroke};
use crate::transpile::collect_handler_idents_excluding_params;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// One compiled `keys` entry.
pub(crate) struct WebKeyArm {
    /// Shadow clones the handler needs (component state/globals it
    /// references) — emitted once per component, ahead of the listener.
    pub shadows: Vec<TokenStream>,
    /// The `if` arm (or a `compile_error!` for a malformed keystroke).
    pub arm: TokenStream,
}

/// Lowers one `keys` entry into a matcher arm over the listener's
/// `quoin::keys::KeyEvent` snapshot (`__quoin_ev`).
///
/// `state_globals` holds the component's state-signal and global names so
/// only those references are shadow-cloned (cloning a plain function name
/// would fail to compile).
#[must_use]
pub(crate) fn lower_key_entry(
    keystroke: &str,
    handler: &Expr,
    state_globals: &std::collections::HashSet<String>,
) -> WebKeyArm {
    let parsed = match ParsedKeystroke::parse(keystroke) {
        Ok(parsed) => parsed,
        Err(why) => {
            let message = format!("invalid `keys` keystroke `{keystroke}`: {why}");
            return WebKeyArm {
                shadows: Vec::new(),
                arm: quote! { ::core::compile_error!(#message) },
            };
        }
    };

    // Handler calling convention (shared with the other backends):
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

    let ctrl = parsed.flags.has(KeyFlags::CTRL);
    let shift = parsed.flags.has(KeyFlags::SHIFT);
    let alt = parsed.flags.has(KeyFlags::ALT);
    let meta = parsed.flags.has(KeyFlags::META);
    // The exact-modifier mask as `quoin::keys::Modifiers` tokens — the web
    // counterpart of the Floem emitter's native `Modifiers` mask.
    let mut mask: Option<TokenStream> = None;
    for (held, flag) in [
        (ctrl, quote! { ::quoin::keys::Modifiers::CTRL }),
        (shift, quote! { ::quoin::keys::Modifiers::SHIFT }),
        (alt, quote! { ::quoin::keys::Modifiers::ALT }),
        (meta, quote! { ::quoin::keys::Modifiers::META }),
    ] {
        if held {
            mask = Some(match mask {
                Some(prev) => quote! { #prev | #flag },
                None => flag,
            });
        }
    }
    let modifiers = mask.unwrap_or_else(|| quote! { ::quoin::keys::Modifiers::EMPTY });
    let dom_key = parsed.dom_key.as_str();
    let arm = quote! {
        if ::quoin::keys::keystroke_matches(__quoin_ev, #modifiers, #dom_key) {
            #call
        }
    };
    WebKeyArm { shadows, arm }
}

/// The shadows plus `if/else`-chained arms for a whole `keys` section.
pub(crate) struct WebKeysBody {
    /// Shadow clones for every handler's state/global references.
    pub shadows: Vec<TokenStream>,
    /// One arm per keystroke, joined with `else` by the caller.
    pub arms: Vec<TokenStream>,
}

/// Lowers a whole `keys` section (one arm per entry). Shadow clones are
/// deduplicated across entries: two handlers referencing the same signal
/// share one clone ahead of the listener.
#[must_use]
pub(crate) fn lower_keys(
    keys: &[crate::parse::KeyEntry],
    state_globals: &std::collections::HashSet<String>,
) -> WebKeysBody {
    let mut shadows = Vec::new();
    let mut shadowed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut arms = Vec::new();
    for key in keys {
        let arm = lower_key_entry(&key.keystroke, &key.handler, state_globals);
        for shadow in arm.shadows {
            // The shadow statement is `let <ident> = clone(&<ident>);` —
            // dedupe by the ident it binds (the token after `let`).
            if let Some(ident) = shadow.clone().into_iter().next().map(|t| t.to_string()) {
                if shadowed.insert(ident) {
                    shadows.push(shadow);
                }
            } else {
                shadows.push(shadow);
            }
        }
        arms.push(arm.arm);
    }
    WebKeysBody { shadows, arms }
}

/// Assembles the whole `keys { .. }` statement block a web `component!`
/// emits: shadow clones, one `attach_window_keydown` listener whose closure
/// carries the `if/else`-chained matcher arms, and the framework's cleanup
/// statement (`cleanup`) disposing the listener handle. Empty for
/// components without a `keys` section.
#[must_use]
pub(crate) fn component_keys_block(
    keys: &[crate::parse::KeyEntry],
    state_globals: &std::collections::HashSet<String>,
    cleanup: &TokenStream,
) -> TokenStream {
    if keys.is_empty() {
        return quote! {};
    }
    let body = lower_keys(keys, state_globals);
    let shadows = &body.shadows;
    let arms = &body.arms;
    quote! {
        {
            #(#shadows)*
            let __quoin_keys_handle = ::quoin::keys::attach_window_keydown(
                move |__quoin_ev| {
                    #(#arms)else*
                },
            );
            #cleanup
        }
    }
}

/// The component's state-signal and global names: the bindings a handler
/// may reference and therefore shadow-clone (cloning a plain function name
/// would fail to compile, so only these qualify).
#[must_use]
pub(crate) fn state_globals_set(
    state: impl Iterator<Item = String>,
    globals: impl Iterator<Item = String>,
) -> std::collections::HashSet<String> {
    state.chain(globals).collect()
}
