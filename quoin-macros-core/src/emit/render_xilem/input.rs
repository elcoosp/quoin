//! `input`/`textarea` for Xilem: `text_input(contents, on_changed)`.
//!
//! The contents string is re-computed by the app logic on every rebuild, so
//! there is no buffer to bridge: the emitter reads the quoin signal once per
//! rebuild and writes edits straight back through the callback.

use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{handler, style};

pub(crate) fn emit_input(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    let value_fallback: syn::Expr =
        syn::parse_quote!((|| unreachable!("input requires `value:`"))());
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);

// No shadow clones for the value handle: cloning the caller's binding
// directly keeps the original state local *used* (shadow clones would
// orphan it and trip `unused_variables` on the component's state field).

    // The on_changed callback: writes the edit into the quoin signal and
    // invokes the user handler with the quoin `Input` event.
    let (handler_setup, call) = match find_arg_expr(el, "on_input") {
        Some(handler_expr) => {
            let (block, name) = handler::handler_block(handler_expr, ctx);
            let call = if handler::handler_arity_is_zero(handler_expr) {
                quote! { #name(); }
            } else {
                quote! { #name(::quoin::ev::Input::new(__new.as_str())); }
            };
            (block, call)
        }
        None => (quote! {}, quote! {}),
    };

    let placeholder = match find_arg_string(el, "placeholder") {
        Some(text) => quote! { .placeholder(#text) },
        None => quote! {},
    };

    let disabled = match find_arg_expr(el, "disabled") {
        Some(expr) => quote! { .disabled(#expr) },
        None => quote! {},
    };

    let base = quote! {
        {
            let __value = ::core::clone::Clone::clone(&(#value_expr));
            #handler_setup
            let __cb_value = ::core::clone::Clone::clone(&__value);
            ::xilem::view::text_input(
                ::quoin::Signal::get(&__value),
                move |__s, __new: String| {
                    let _ = __s;
                    ::quoin::Signal::set(&__cb_value, __new.clone());
                    #call
                },
            )#placeholder #disabled
        }
    };

    match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            let mut out = base;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => base,
    }
}
