//! `input`/`textarea` for Floem: `text_input` bound to a quoin signal.
//!
//! Floem's `text_input` owns its edits through a native `RwSignal<String>`.
//! The quoin signal lives on the other side of the boundary, so the emitter
//! creates the buffer and two guard effects:
//!
//! 1. quoin → floem: push external value changes into the buffer,
//! 2. floem → quoin: push user edits back into the quoin signal and invoke
//!    `on_input` with the quoin [`Input`](quoin_core::ev::Input) event.
//!
//! Both effects compare before writing, so an edit round-trips exactly once
//! and there is no feedback loop.

use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{handler, style};

pub(crate) fn emit_input(el: &Element, ctx: &mut super::EmitterCtx) -> TokenStream {
    // `check_element_args` rejects `input` without `value`; the fallback
    // below fails loudly if it ever runs.
    let value_fallback: syn::Expr = syn::parse_quote!((|| unreachable!("input requires `value:`")));
    let value_expr = find_arg_expr(el, "value").unwrap_or(&value_fallback);

    let value_capture = handler::capture_shadows(value_expr, ctx);

    // The on_input handler, pre-bound outside the reactive effect.
    let on_input_setup = match find_arg_expr(el, "on_input") {
        Some(handler_expr) => {
            let (block, _name) = handler::handler_block(handler_expr, ctx);
            let call = if handler::handler_arity_is_zero(handler_expr) {
                quote! { __h() }
            } else {
                quote! { __h(::quoin::ev::Input::new(__edited)) }
            };
            quote! {
                #block
                ::floem::reactive::Effect::new(
                    move |__prev: ::core::option::Option<()>| {
                        let _ = __prev;
                        let __edited =
                            ::floem::reactive::SignalGet::get(&__buffer);
                        #call;
                    },
                );
            }
        }
        None => quote! {},
    };

    let input_view = if el.name == "textarea" {
        // Multi-line editing maps onto Floem's text editor; the buffer still
        // round-trips through the same two effects.
        quote! { ::floem::views::text_editor(::floem::imbl::Rope::from(
            ::floem::reactive::SignalGet::get_untracked(&__buffer),
        )) }
    } else {
        quote! { ::floem::views::TextInput::new(__buffer) }
    };

    let placeholder = match crate::emit::common::find_arg_string(el, "placeholder") {
        Some(text) => quote! { .placeholder(#text) },
        None => quote! {},
    };

    let styled = match crate::emit::common::find_arg_string(el, "class") {
        Some(class) => {
            let styles = style::lower_class(&class);
            style::apply_styles(input_view, &styles)
        }
        None => input_view,
    };

    let disabled = match find_arg_expr(el, "disabled") {
        Some(expr) => quote! { .disabled(move || #expr) },
        None => quote! {},
    };

    quote! {
        {
            #(#value_capture)*
            // Clone the handle (signals are cheap handles); a field path like
            // `store.draft` must not move the field out of the store.
            let __value = ::core::clone::Clone::clone(&(#value_expr));
            let __buffer = ::floem::reactive::RwSignal::new(::quoin::Signal::get(&__value));
            // quoin → floem: external writes land in the buffer.
            ::floem::reactive::Effect::new(move |__prev: ::core::option::Option<()>| {
                let _ = __prev;
                let __next = ::quoin::Signal::get(&__value);
                if ::floem::reactive::SignalGet::get_untracked(&__buffer) != __next {
                    ::floem::reactive::SignalUpdate::set(&__buffer, __next);
                }
            });
            #on_input_setup
            #styled #placeholder #disabled
        }
    }
}
