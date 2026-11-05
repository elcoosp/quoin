//! `button` for Floem: `button(child)` + the `action` decorator.
//!
//! Two lowerings, switched by the `floem-shadcn` feature:
//!
//! - **shadcn** (feature on): `floem_shadcn::components::button::Button`
//!   with the canonical `variant:`/`size:` grammar (`emit::shadcn_spec`).
//! - **generic** (feature off): Floem's native `Button` — the v1 contract,
//!   unchanged.
//!
//! `button(label: "…")` lowers to `Button::new(label(|| "…"))`; a children
//! block lowers to `Button::new(<child view>)`. The handler is moved
//! directly (no `Rc`/`Arc`), matching the zero-overhead contract.

// Imports vary with the `floem-shadcn` feature; a file-level allow keeps
// every feature combination compiling without per-line noise.
#![allow(unused_imports)]

use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, generic, handler, style};

pub(crate) fn emit_button(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "floem-shadcn")]
    {
        // Canonical `variant:`/`size:` — unknown values are hard errors.
        let variant = crate::emit::shadcn_spec::canonical_button_variant(el);
        if let Some(error) =
            variant.as_ref().and_then(crate::emit::shadcn_spec::choice_error)
        {
            return error;
        }
        let size = crate::emit::shadcn_spec::canonical_button_size(el);
        if let Some(error) = size.as_ref().and_then(crate::emit::shadcn_spec::choice_error) {
            return error;
        }

        // The button's child: the `label:` argument (literal or expression),
        // or the children block.
        let child: TokenStream = if let Some(label) = find_arg_string(el, "label") {
            let lit = syn::LitStr::new(&label, proc_macro2::Span::call_site());
            quote! { ::floem::views::Label::derived(move || #lit) }
        } else if let Some(label_expr) = crate::emit::common::find_arg_expr(el, "label") {
            let shadows = super::handler::capture_shadows(label_expr, ctx);
            quote! { { #(#shadows)* ::floem::views::Label::derived(move || #label_expr) } }
        } else {
            let mut kids = Vec::new();
            for child in &el.children {
                kids.push(emit_node(child, ctx, inside_for));
            }
            match kids.len() {
                0 => quote! { ::floem::views::Empty::new() },
                1 => kids[0].clone(),
                _ => generic::static_stack(&kids, quoin_style::FlexDir::Row, None),
            }
        };

        let variant_chain = match variant.and_then(|c| c.ok()) {
            Some("destructive") => {
                quote! { .with_variant(::floem_shadcn::components::button::ButtonVariant::Destructive) }
            }
            Some("outline") => {
                quote! { .with_variant(::floem_shadcn::components::button::ButtonVariant::Outline) }
            }
            Some("secondary") => {
                quote! { .with_variant(::floem_shadcn::components::button::ButtonVariant::Secondary) }
            }
            Some("ghost") => {
                quote! { .with_variant(::floem_shadcn::components::button::ButtonVariant::Ghost) }
            }
            Some("link") => {
                quote! { .with_variant(::floem_shadcn::components::button::ButtonVariant::Link) }
            }
            // `default`/`primary`/unspecified — the shadcn default look.
            _ => quote! {},
        };
        let size_chain = match size.and_then(|c| c.ok()) {
            Some("sm") => {
                quote! { .with_size(::floem_shadcn::components::button::ButtonSize::Sm) }
            }
            Some("lg") => {
                quote! { .with_size(::floem_shadcn::components::button::ButtonSize::Lg) }
            }
            Some("icon") => {
                quote! { .with_size(::floem_shadcn::components::button::ButtonSize::Icon) }
            }
            _ => quote! {},
        };

        let chain = quote! {
            ::floem_shadcn::components::button::Button::new(#child) #variant_chain #size_chain
        };

        let styled = match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(chain, &styles)
            }
            None => chain,
        };

        let action = match crate::emit::common::find_arg_expr(el, "on_click") {
            Some(handler_expr) => handler::handler_to_action(handler_expr, ctx),
            None => quote! {},
        };

        let disabled = match crate::emit::common::find_arg_expr(el, "disabled") {
            Some(expr) => quote! { .disabled(move || #expr) },
            None => quote! {},
        };

        quote! { #styled #action #disabled }
    }

    #[cfg(not(feature = "floem-shadcn"))]
    {
        // The button's child: the `label:` argument (literal or expression),
        // or the children block.
        let child: TokenStream = if let Some(label) = find_arg_string(el, "label") {
            let lit = syn::LitStr::new(&label, proc_macro2::Span::call_site());
            quote! { ::floem::views::Label::derived(move || #lit) }
        } else if let Some(label_expr) = crate::emit::common::find_arg_expr(el, "label") {
            // The label expression is the *value*: wrap it in the derived
            // label's closure so it re-evaluates reactively.
            let shadows = super::handler::capture_shadows(label_expr, ctx);
            quote! { { #(#shadows)* ::floem::views::Label::derived(move || #label_expr) } }
        } else {
            let mut kids = Vec::new();
            for child in &el.children {
                kids.push(emit_node(child, ctx, inside_for));
            }
            match kids.len() {
                0 => quote! { ::floem::views::Empty::new() },
                1 => kids[0].clone(),
                _ => generic::static_stack(&kids, quoin_style::FlexDir::Row, None),
            }
        };

        let chain = quote! { ::floem::views::Button::new(#child) };

        let styled = match find_arg_string(el, "class") {
            Some(class) => {
                let styles = style::lower_class(&class);
                style::apply_styles(chain, &styles)
            }
            None => chain,
        };

        // The handler: `on_click` (M4 renames nothing — the arg is the same on
        // every backend).
        let action = match crate::emit::common::find_arg_expr(el, "on_click") {
            Some(handler_expr) => handler::handler_to_action(handler_expr, ctx),
            None => quote! {},
        };

        let disabled = match crate::emit::common::find_arg_expr(el, "disabled") {
            Some(expr) => quote! { .disabled(move || #expr) },
            None => quote! {},
        };

        quote! { #styled #action #disabled }
    }
}
