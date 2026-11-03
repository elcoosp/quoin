//! Generic containers for the Xilem backend: `flex(Axis, (a, b, c))` with a
//! `sized_box` wrap for box-model styles.

use crate::emit::common::find_arg_string;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_node, style};
use quoin_style::FlexDir;

/// A flex view-sequence tuple (chunked when over the tuple arity).
pub(crate) fn flex_sequence(views: &[TokenStream], dir: FlexDir) -> TokenStream {
    let axis = match dir {
        FlexDir::Column | FlexDir::ColumnReverse => quote! { ::xilem::kurbo::Axis::Vertical },
        _ => quote! { ::xilem::kurbo::Axis::Horizontal },
    };
    let inner = if views.len() == 1 {
        let first = &views[0];
        quote! { (#first,) }
    } else {
        quote! { (#(#views,)* ) }
    };
    quote! { ::xilem::view::flex(#axis, #inner) }
}

fn chunked_flex(views: &[TokenStream], dir: FlexDir) -> TokenStream {
    if views.len() <= super::MAX_TUPLE_ARITY {
        return flex_sequence(views, dir);
    }
    let chunks: Vec<TokenStream> = views
        .chunks(super::MAX_TUPLE_ARITY)
        .map(flex_sequence_chunk)
        .collect();
    flex_sequence(&chunks, dir)
}

/// One chunk of the chunked flex: `flex_sequence` over a slice.
fn flex_sequence_chunk(chunk: &[TokenStream]) -> TokenStream {
    flex_sequence(chunk, FlexDir::Column)
}

/// The default heading/paragraph text size, matching the GPUI backend.
fn heading_class(tag: &str) -> Option<&'static str> {
    match tag {
        "h1" => Some("text-2xl"),
        "h2" => Some("text-xl"),
        "h3" => Some("text-lg"),
        _ => None,
    }
}

/// A text-leaf element (`h1`, `p`, `span`, …): a styled label with the
/// cascaded text context.
fn emit_text_element(el: &Element, tag: &str, ctx: &mut super::EmitterCtx) -> TokenStream {
    let class = find_arg_string(el, "class").or_else(|| heading_class(tag).map(str::to_owned));
    let saved = ctx.text.clone();
    let mut errors: Vec<TokenStream> = Vec::new();
    if let Some(class) = &class {
        let mut text = saved.clone();
        let lowered = style::lower_class(class, &mut text);
        errors = lowered.errors;
        ctx.text = text;
    }

    let mut tokens = Vec::new();
    for child in &el.children {
        match child {
            RenderNode::Text(lit) => {
                tokens.push(super::emit_node(&RenderNode::Text(lit.clone()), ctx, false));
            }
            RenderNode::Expr(expr) => tokens.push(super::emit_node(
                &RenderNode::Expr(expr.clone()),
                ctx,
                false,
            )),
            other => tokens.push(emit_node(other, ctx, false)),
        }
    }
    ctx.text = saved;

    let out = if tokens.len() > 1 {
        flex_sequence(&tokens, FlexDir::Column)
    } else if tokens.is_empty() {
        quote! { ::xilem::view::sized_box(::xilem::view::label("")) }
    } else {
        tokens[0].clone()
    };
    if errors.is_empty() {
        out
    } else {
        quote! { { #(#errors)* #out } }
    }
}

/// Container element lowering for `div` and friends.
pub(crate) fn emit_container(
    el: &Element,
    ctx: &mut super::EmitterCtx,
    inside_for: bool,
) -> TokenStream {
    let _ = inside_for;
    let tag = el.name.to_string();

    if matches!(
        tag.as_str(),
        "h1" | "h2" | "h3" | "p" | "text" | "span" | "label"
    ) {
        return emit_text_element(el, &tag, ctx);
    }

    let class = find_arg_string(el, "class");
    let saved_text = ctx.text.clone();
    let styles = class
        .as_deref()
        .map(|c| style::lower_class(c, &mut ctx.text));

    // Dynamic children: `div(children: expr)`.
    if let Some(children) = crate::emit::common::find_arg_expr(el, "children") {
        let axis = if matches!(
            styles.as_ref().map(style::flex_dir),
            Some(FlexDir::Column | FlexDir::ColumnReverse)
        ) {
            quote! { ::xilem::kurbo::Axis::Vertical }
        } else {
            quote! { ::xilem::kurbo::Axis::Horizontal }
        };
        let flex_tokens: Vec<TokenStream> =
            styles.as_ref().map_or_else(Vec::new, |s| s.flex.clone());
        let base = quote! {
            ::xilem::view::flex(
                #axis,
                ::std::vec::Vec::from_iter(
                    (#children).into_iter().map(|__v| __v)
                ),
            )#(#flex_tokens)*
        };
        ctx.text = saved_text;
        let styled_base = apply_box_styles(base, styles.as_ref());
        return apply_click(el, styled_base, ctx);
    }

    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, false));
    }
    let kid_count = kids.len();
    ctx.text = saved_text;

    let dir = styles.as_ref().map_or(FlexDir::Row, style::flex_dir);
    let has_box = styles.as_ref().is_some_and(|s| !s.box_chain.is_empty());

    let out = if kid_count == 0 {
        // An empty container is a zero-sized flex with no children.
        let empty = quote! { ::xilem::view::sized_box(::xilem::view::label("")) };
        apply_box_styles(empty, styles.as_ref())
    } else if kid_count == 1 && !has_box {
        // Single child keeps its own type; diagnostics still wrap it.
        let first = kids.remove(0);
        match &styles {
            Some(s) => style::wrap_errors(first, s),
            None => first,
        }
    } else {
        let flex_tokens: Vec<TokenStream> =
            styles.as_ref().map_or_else(Vec::new, |s| s.flex.clone());
        let flex = chunked_flex(&kids, dir);
        let base = quote! { #flex #(#flex_tokens)* };
        apply_box_styles(base, styles.as_ref())
    };

    apply_click(el, out, ctx)
}

/// Applies the sized-box chain (box-model styles) to a view, wrapping in
/// `sized_box` when needed, plus the unknown-class errors.
fn apply_box_styles(base: TokenStream, styles: Option<&style::XilemStyles>) -> TokenStream {
    match styles {
        None => base,
        Some(s) => {
            let mut out = base;
            if !s.box_chain.is_empty() {
                let chain = &s.box_chain;
                out = quote! {
                    ::xilem::view::sized_box(#out)#(#chain)*
                };
            }
            style::wrap_errors(out, s)
        }
    }
}

/// Applies `on_click` to a Xilem container by wrapping it in a `button`.
fn apply_click(el: &Element, view: TokenStream, ctx: &mut super::EmitterCtx) -> TokenStream {
    if let Some(handler) = crate::emit::common::find_arg_expr(el, "on_click") {
        let action = super::handler::handler_to_button_cb(handler, ctx);
        return quote! { ::xilem::view::button(#view, #action) };
    }
    view
}
