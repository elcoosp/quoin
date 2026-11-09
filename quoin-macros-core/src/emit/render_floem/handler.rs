//! Handler lowering for the Floem backend.
//!
//! A quoin handler (`on_click: |_| …`) becomes a scoped block that
//! shadow-clones the signals the body touches and stores the user closure in
//! `__h`; the framework-facing closure calls it with the quoin event payload
//! (`()` in M2, `quoin::ev::Click` from M4):
//!
//! ```ignore
//! {
//!     let count = ::core::clone::Clone::clone(&count);
//!     let __h = (move |_| count.update(|c| *c += 1));
//!     button(label("+")).action(move || __h(()))
//! }
//! ```
//!
//! Handlers are moved directly — no `Rc`/`Arc` wrapper. The zero-parameter
//! form (`|| …`) stays valid: the adapter calls `__h()` instead.

use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// The shadow-clone block + user closure for a quoin handler expression.
///
/// Returns the tokens for `{ shadows; let __h = handler; }`; the caller
/// appends the framework-facing closure. Idents referenced by the handler
/// are also registered with `ctx`, so the expansion hoists one clone of
/// each outer variable ahead of every `move` closure.
pub(crate) fn handler_block(
    handler: &Expr,
    ctx: &mut super::EmitterCtx,
) -> (TokenStream, proc_macro2::Ident) {
    let shadows = capture_shadows(handler, ctx);
    let handler_move = force_move_on_closure(handler);
    (
        quote! {
            #(#shadows)*
            let __h = (#handler_move);
        },
        quote::format_ident!("__h"),
    )
}

/// The signal shadow-clones a handler needs, one `let` per ident.
///
/// Idents in `ctx.excluded` (loop patterns introduced by the macro itself)
/// are skipped: they are bound by the enclosing closure's parameters, not
/// by the caller's scope.
pub(crate) fn capture_shadows(handler: &Expr, ctx: &mut super::EmitterCtx) -> Vec<TokenStream> {
    let idents = collect_handler_idents_excluding_params(handler);
    let mut shadows = Vec::new();
    for id in idents {
        if ctx.excluded.contains(&id.to_string()) {
            continue;
        }
        ctx.capture(&id);
        shadows.push(quote! { let #id = ::core::clone::Clone::clone(&#id); });
    }
    shadows
}

/// Whether the user handler takes an event parameter: defined once in
/// [`crate::emit::common`] so the web backends share the same calling
/// convention.
pub(crate) use crate::emit::common::handler_arity_is_zero;

/// Lowers `on_click` into the Floem `.action(move || __h(payload))`
/// decorator, wrapped in the capture block.
#[must_use]
pub(crate) fn handler_to_action(handler: &Expr, ctx: &mut super::EmitterCtx) -> TokenStream {
    let (block, name) = handler_block(handler, ctx);
    let call = if handler_arity_is_zero(handler) {
        quote! { #name() }
    } else {
        quote! { #name(()) }
    };
    quote! {
        .action({
            #block
            move || #call
        })
    }
}

/// The general event-handler form for decorators that receive an event:
/// `.on_event(listener, move |__cx, __event| __h(convert(__event)))`.
/// Unused in M2 (`action` covers clicks); kept for the M5 keys/events push.
#[expect(dead_code, reason = "wired up by the M5 typed-event push")]
pub(crate) fn handler_to_on_event(
    handler: &Expr,
    ctx: &mut super::EmitterCtx,
    listener: &TokenStream,
    convert: &TokenStream,
) -> TokenStream {
    let (block, name) = handler_block(handler, ctx);
    let call = if handler_arity_is_zero(handler) {
        quote! { #name() }
    } else {
        quote! { #name(#convert) }
    };
    quote! {
        .on_event(#listener, {
            #block
            move |__cx, __event| {
                let _ = __cx;
                #call
            }
        })
    }
}

/// Wraps a closure in `{ shadows; closure }` so `Fn` closures capture their
/// own clones instead of moving the caller's bindings. Shared by the
/// `for`/`if` lowering and the handler decorators.
pub(crate) fn wrap_closure_with(shadows: &[TokenStream], closure: TokenStream) -> TokenStream {
    if shadows.is_empty() {
        closure
    } else {
        quote! { { #(#shadows)* #closure } }
    }
}
