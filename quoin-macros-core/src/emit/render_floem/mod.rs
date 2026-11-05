//! Floem backend for `quoin_render!`.
//!
//! The emitter walks the [`RenderNode`] tree and produces Floem view
//! constructors (`label`, `button`, `v_stack`, `dyn_stack`, …) with
//! compile-time-resolved [`Style`] chains. Every static child list becomes a
//! Floem `ViewTuple` — a tuple, never a `Vec<Box<dyn View>>` — so the
//! zero-overhead contract holds: boxing appears only at dynamic seams
//! (`if`/`for`), where a hand-written Floem application pays the same cost.
//!
//! # Layout mapping
//!
//! `div` (and the other container tags) lower to `v_stack`/`h_stack`/`stack`
//! depending on the parsed `flex-col`/`flex-row` decls (row is the default,
//! matching both Tailwind's `flex` and the GPUI backend). A single child
//! lowers to `container`, no children to `empty`.
//!
//! # Text
//!
//! Text leaves lower to `label`. Static literals keep their allocation-free
//! `&'static str`; expressions become `move || expr` closures re-evaluated by
//! Floem's reactivity. Text style decls (`text-*`, `font-*`) are emitted on
//! the label itself *and* cascaded onto containers, where Floem's inherited
//! style props propagate them to non-label text.

use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quoin_style::FlexDir;
use quote::quote;

/// Emission state threaded through the walker: hoisted `let` bindings and
/// the set of outer variables every emitted closure captures (cloned once at
/// the top of the expansion, so no `move` closure can move the caller's
/// original binding) plus the loop patterns that must *not* be hoisted.
pub(crate) struct EmitterCtx {
    /// `let` statements hoisted ahead of the view expression.
    pub bindings: Vec<TokenStream>,
    /// Outer variable idents captured somewhere in the tree.
    pub captured: Vec<proc_macro2::Ident>,
    /// Loop patterns (and other locals introduced by the macro itself) that
    /// the hoist must skip.
    pub excluded: std::collections::HashSet<String>,
}

impl EmitterCtx {
    pub(crate) fn new() -> Self {
        Self {
            bindings: Vec::new(),
            captured: Vec::new(),
            excluded: std::collections::HashSet::new(),
        }
    }

    /// Registers a captured ident (deduplicated, skipping loop patterns).
    pub(crate) fn capture(&mut self, ident: &proc_macro2::Ident) {
        if !self.excluded.contains(&ident.to_string())
            && !self.captured.iter().any(|id| id == ident)
        {
            self.captured.push(ident.clone());
        }
    }
}

mod button;
pub mod component;
mod control_flow;
mod generic;
mod handler;
mod input;
mod keys;
mod style;
mod text;
mod widgets;

#[must_use]
/// Expands a `quoin_render!` AST into a Floem view expression.
pub fn emit_render(node: &RenderNode) -> TokenStream {
    let mut ctx = EmitterCtx::new();
    let inner = emit_node(node, &mut ctx, false);

    // One clone per captured outer variable, hoisted ahead of every `move`
    // closure: the closures capture the clone, and per-site shadow clones
    // (inside handlers/labels) re-clone the handle per invocation.
    let mut hoists = ctx
        .captured
        .iter()
        .map(|id| {
            quote! { let #id = ::core::clone::Clone::clone(&#id); }
        })
        .collect::<Vec<_>>();
    hoists.extend(ctx.bindings);

    let tokens = if hoists.is_empty() {
        quote! { { #inner } }
    } else {
        quote! { { #(#hoists)* #inner } }
    };

    wrap_with_cfg(node.attrs(), tokens)
}

/// The maximum arity of Floem's `ViewTuple` impls; longer static child lists
/// are chunked into nested same-direction stacks (still no boxing).
pub(crate) const MAX_TUPLE_ARITY: usize = 16;

pub(crate) fn emit_node(node: &RenderNode, ctx: &mut EmitterCtx, inside_for: bool) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el, ctx, inside_for),
        RenderNode::Text(t) => text::emit_static_text(t, None),
        RenderNode::Expr(e) => text::emit_dynamic_text(e, None, ctx),
        RenderNode::If(if_node) => control_flow::emit_if(if_node, ctx),
        RenderNode::For(for_node) => control_flow::emit_for(for_node, ctx),
        RenderNode::Root(nodes) => {
            // Floem has no fragment: a multi-node root becomes a plain
            // column stack, matching how a body tag lays out block children.
            let views: Vec<TokenStream> = nodes
                .iter()
                .map(|n| emit_node(n, ctx, inside_for))
                .collect();
            if views.len() == 1 {
                views[0].clone()
            } else {
                generic::static_stack(&views, FlexDir::Column, None)
            }
        }
    }
}

fn emit_element(el: &Element, ctx: &mut EmitterCtx, inside_for: bool) -> TokenStream {
    let inner = emit_element_inner(el, ctx, inside_for);
    wrap_with_cfg(&el.attrs, inner)
}

fn emit_element_inner(el: &Element, ctx: &mut EmitterCtx, inside_for: bool) -> TokenStream {
    let name_str = el.name.to_string();
    match name_str.as_str() {
        "button" => button::emit_button(el, ctx, inside_for),
        "input" | "textarea" => input::emit_input(el, ctx),
        "separator" => widgets::emit_separator(el),
        // M5 stateful widgets. Each either has a native lowering or emits a
        // precise `compile_error!` — silence is never an option.
        "checkbox" => widgets::emit_checkbox(el, ctx),
        "switch" => widgets::emit_switch(el, ctx),
        "slider" => widgets::emit_slider(el, ctx),
        "tooltip" => widgets::emit_tooltip(el, ctx, inside_for),
        "scroll_area" => widgets::emit_scroll_area(el, ctx, inside_for),
        "virtual_list" => widgets::emit_virtual_list(el, ctx),
        "tabs" | "tab_bar" => widgets::emit_tabs(el, ctx, inside_for),
        "data_table" => widgets::emit_data_table(el, ctx),
        "dropdown_menu" => widgets::emit_dropdown_menu(el, ctx),
        "rich_text" => widgets::unsupported_element(
            &name_str,
            "rich text is not on the v1 Floem parity path; use `text` with styled `span`s",
        ),
        "icon" => widgets::emit_icon(el),
        "badge" => widgets::emit_badge(el, ctx, inside_for),
        "progress" => widgets::emit_progress(el, ctx),
        "skeleton" => widgets::emit_skeleton(el),
        "skeleton_text" => widgets::emit_skeleton_text(el),
        "skeleton_avatar" => widgets::emit_skeleton_avatar(el),
        "clipboard_button" => widgets::emit_clipboard_button(el, ctx),
        "styled_text" => widgets::unsupported_element(
            &name_str,
            "styled_text is not on the v1 Floem parity path",
        ),
        "radio_group" | "radio" => widgets::unsupported_element(
            &name_str,
            "radio inputs land with the M5 widget push; use `checkbox` meanwhile",
        ),
        // Rich-elements Phase 1: canonical shadcn Tier-1 surface. Each
        // emitter switches between the `floem-shadcn` component and the
        // generic container fallback; `alert_title`/`alert_description`/
        // `avatar_fallback` stay generic surface elements on Floem (their
        // parents carry the copy as args).
        "card" => widgets::emit_card(el, ctx, inside_for),
        "card_header" => widgets::emit_card_header(el, ctx, inside_for),
        "card_title" | "card_description" | "card_content" | "card_footer"
        | "card_action" => widgets::emit_card_part(el, ctx, inside_for),
        "alert" => widgets::emit_alert(el, ctx, inside_for),
        "spinner" => widgets::emit_spinner(el, ctx, inside_for),
        "kbd" => widgets::emit_kbd(el, ctx, inside_for),
        "avatar" => widgets::emit_avatar(el, ctx, inside_for),
        "img" | "svg" => widgets::unsupported_element(
            &name_str,
            "images/svg are not on the v1 Floem parity path",
        ),
        _ => generic::emit_container(el, ctx, inside_for),
    }
}
