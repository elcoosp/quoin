//! Generic containers (`div`, `p`, `h1`… parents, `form`, …) for Floem.
//!
//! Static children become a native tuple — `v_stack((a, b, c))` — never a
//! `Vec<Box<dyn View>>`. Long lists (over [`MAX_TUPLE_ARITY`]) are chunked
//! into nested same-direction stacks, which preserves both the layout and
//! the tuple discipline. Exactly one child lowers to `container`, zero
//! children to `empty()`.
//!
//! Text properties cascade through Floem's inherited style props
//! (`color`, `font_size`, `font_weight` are `{ inherited }`), so a container
//! with `text-muted-foreground` colors every nested label without the
//! emitter touching the label at all.

use crate::emit::common::find_arg_string;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::{MAX_TUPLE_ARITY, emit_node};
use quoin_style::FlexDir;

use super::style;

/// The direction-aware static stack constructor for a child tuple.
pub(crate) fn static_stack(
    views: &[TokenStream],
    dir: FlexDir,
    style_chain: Option<TokenStream>,
) -> TokenStream {
    let ctor = match dir {
        FlexDir::Column | FlexDir::ColumnReverse => quote! { ::floem::views::Stack::vertical },
        _ => quote! { ::floem::views::Stack::horizontal },
    };
    let inner = if views.len() == 1 {
        let first = &views[0];
        quote! { (#first,) }
    } else {
        quote! { (#(#views,)* ) }
    };
    let stack = quote! { #ctor(#inner) };
    match style_chain {
        Some(chain) => quote! { #stack #chain },
        None => stack,
    }
}

/// Chunks a static child list into nested same-direction stacks when it
/// exceeds the `ViewTuple` arity.
fn chunked_stack(views: &[TokenStream], dir: FlexDir) -> TokenStream {
    if views.len() <= MAX_TUPLE_ARITY {
        return static_stack(views, dir, None);
    }
    let chunks: Vec<TokenStream> = views
        .chunks(MAX_TUPLE_ARITY)
        .map(|chunk| static_stack(chunk, dir, None))
        .collect();
    // Parent stack of chunks; the direction repeats so nesting is uniform.
    static_stack(&chunks, dir, None)
}

/// The default heading/paragraph text size, matching the GPUI backend's
/// built-in heading treatment.
fn heading_class(tag: &str) -> Option<&'static str> {
    match tag {
        "h1" => Some("text-2xl"),
        "h2" => Some("text-xl"),
        "h3" => Some("text-lg"),
        _ => None,
    }
}

/// A text-leaf element (`h1`, `p`, `span`, `text`, `label`): a styled `label`.
fn emit_text_element(el: &Element, tag: &str, ctx: &mut super::EmitterCtx) -> TokenStream {
    let class = find_arg_string(el, "class").or_else(|| heading_class(tag).map(str::to_owned));
    let text_style = class.as_deref().map(style::lower_class);

    let mut tokens = Vec::new();
    for child in &el.children {
        match child {
            RenderNode::Text(lit) => {
                let base = super::text::emit_static_text(lit, None);
                let styled = text_style
                    .as_ref()
                    .map(|s| style::apply_styles(base.clone(), s))
                    .unwrap_or(base);
                tokens.push(styled);
            }
            RenderNode::Expr(expr) => {
                let base = super::text::emit_dynamic_text(expr, None, ctx);
                let styled = text_style
                    .as_ref()
                    .map(|s| style::apply_styles(base.clone(), s))
                    .unwrap_or(base);
                tokens.push(styled);
            }
            other => {
                // Nested elements inside a text leaf: lower as usual inside
                // a column stack (matches inline-flow wrapping loosely).
                tokens.push(emit_node(other, ctx, false));
            }
        }
    }

    if tokens.len() > 1 {
        static_stack(&tokens, FlexDir::Column, None)
    } else if tokens.is_empty() {
        quote! { ::floem::views::Empty::new() }
    } else {
        tokens[0].clone()
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
    let stack_style = class.as_deref().map(style::lower_class);

    // Dynamic children: `div(children: expr)`.
    if let Some(children) = crate::emit::common::find_arg_expr(el, "children") {
        let ctor = if let Some(FlexDir::Column | FlexDir::ColumnReverse) =
            stack_style.as_ref().map(style::flex_dir)
        {
            quote! { ::floem::views::Stack::vertical_from_iter }
        } else {
            quote! { ::floem::views::Stack::horizontal_from_iter }
        };
        let base =
            quote! { #ctor((#children).into_iter().map(|__v| __v.into_any()).collect::<Vec<_>>()) };
        let out = stack_style
            .as_ref()
            .map(|s| style::apply_styles(base.clone(), s))
            .unwrap_or(base);
        return apply_click(el, ctx, out);
    }

    let mut kids = Vec::new();
    for child in &el.children {
        kids.push(emit_node(child, ctx, false));
    }
    let kid_count = kids.len();

    let dir = stack_style.as_ref().map_or(FlexDir::Row, style::flex_dir);
    let has_box_styles = stack_style.as_ref().is_some_and(|s| !s.normal.is_empty());
    let base = if kid_count == 0 {
        quote! { ::floem::views::Empty::new() }
    } else if kid_count == 1 && !has_box_styles {
        // Single child without container box styles: keep the child's type.
        kids.remove(0)
    } else if kid_count == 1 {
        let first = kids.remove(0);
        quote! { ::floem::views::Container::new(#first) }
    } else {
        chunked_stack(&kids, dir)
    };

    // Styles (and unknown-class diagnostics) wrap the completed base view.
    let body = stack_style
        .as_ref()
        .map(|s| style::apply_styles(base.clone(), s))
        .unwrap_or(base);

    apply_click(el, ctx, body)
}

/// Applies `on_click` to any Floem view through the `action` decorator
/// (which also covers Enter/Space activation).
fn apply_click(el: &Element, ctx: &mut super::EmitterCtx, view: TokenStream) -> TokenStream {
    if let Some(handler) = crate::emit::common::find_arg_expr(el, "on_click") {
        let action = super::handler::handler_to_action(handler, ctx);
        return quote! { #view #action };
    }
    view
}
