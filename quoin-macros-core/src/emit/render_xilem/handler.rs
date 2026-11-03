//! Handler lowering for the Xilem backend.
//!
//! Xilem callbacks receive `&mut State` and return the app's `Action`; the
//! quoin handler is stored in `__h` and invoked with the quoin event payload
//! (`()` in the current milestones). The closures the app logic rebuilds
//! every frame, so shadow clones run once per rebuild — no stale captures.

use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// The shadow-clone block + user closure for a quoin handler expression.
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

/// Whether the user handler takes an event parameter.
pub(crate) fn handler_arity_is_zero(handler: &Expr) -> bool {
    match handler {
        Expr::Closure(c) => c.inputs.is_empty(),
        _ => false,
    }
}

/// Wraps a closure in `{ shadows; closure }` so `Fn` closures capture their
/// own clones instead of moving the caller's bindings.
pub(crate) fn wrap_closure_with(shadows: &[TokenStream], closure: TokenStream) -> TokenStream {
    if shadows.is_empty() {
        closure
    } else {
        quote! { { #(#shadows)* #closure } }
    }
}

/// Lowers `on_click` into a Xilem button callback:
/// `move |__s| { __h(()) }`. Handlers are moved directly (no `Rc`/`Arc`).
#[must_use]
pub(crate) fn handler_to_button_cb(handler: &Expr, ctx: &mut super::EmitterCtx) -> TokenStream {
    let (block, name) = handler_block(handler, ctx);
    let call = if handler_arity_is_zero(handler) {
        quote! { #name() }
    } else {
        quote! { #name(()) }
    };
    quote! {
        {
            #block
            move |__s| {
                let _ = __s;
                #call
            }
        }
    }
}
