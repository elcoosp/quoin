//! `button` for Xilem: `text_button` for labels, `button(child, callback)`
//! for custom children. Handlers are moved directly (no `Rc`/`Arc`).

use crate::emit::common::find_arg_string;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, generic, handler, style};

pub(crate) fn emit_button(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    // The button's child: the `label:` argument, or the children block.
    let child: TokenStream = if let Some(label) = find_arg_string(el, "label") {
        let lit = syn::LitStr::new(&label, proc_macro2::Span::call_site());
        quote! { #lit }
    } else if let Some(label_expr) = crate::emit::common::find_arg_expr(el, "label") {
        // The label expression is the value; the app logic re-evaluates it
        // on every rebuild.
        let idents = crate::transpile::collect_handler_idents_excluding_params(label_expr);
        for ident in &idents {
            ctx.capture(ident);
        }
        quote! { #label_expr }
    } else {
        let mut kids = Vec::new();
        for child in &el.children {
            kids.push(emit_node(child, ctx, inside_for));
        }
        match kids.len() {
            0 => quote! { "" },
            1 => kids[0].clone(),
            _ => generic::flex_sequence(&kids, quoin_style::FlexDir::Row),
        }
    };

    let handler = if let Some(handler_expr) = crate::emit::common::find_arg_expr(el, "on_click") {
        handler::handler_to_button_cb(handler_expr, ctx)
    } else {
        quote! { move |__s| { let _ = __s; } }
    };

    let base = if el.children.is_empty() {
        // `label:` argument (literal or expression): the styled text button.
        quote! { ::xilem::view::text_button(#child, #handler) }
    } else {
        quote! { ::xilem::view::button(#child, #handler) }
    };

    // Style the button's inner content (text styles cascade to the label
    // inside; masonry buttons carry their own chrome).
    let styled = match find_arg_string(el, "class") {
        Some(class) => {
            let saved = ctx.text.clone();
            let lowered = style::lower_class(&class, &mut ctx.text);
            ctx.text = saved;
            // Box-model styles wrap the button.
            let mut out = base;
            if !lowered.box_chain.is_empty() {
                let chain = &lowered.box_chain;
                out = quote! { ::xilem::view::sized_box(#out)#(#chain)* };
            }
            style::wrap_errors(out, &lowered)
        }
        None => base,
    };

    let disabled = match crate::emit::common::find_arg_expr(el, "disabled") {
        Some(expr) => quote! { .disabled(#expr) },
        None => quote! {},
    };

    quote! { #styled #disabled }
}
