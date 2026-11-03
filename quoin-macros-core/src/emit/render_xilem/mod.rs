//! Xilem backend for `quoin_render!`.
//!
//! Xilem rebuilds the whole view from the app-logic function after every
//! action, so the emitter produces one concrete view expression per
//! expansion — plain `flex`/`sized_box`/`label` builders, no signals in the
//! tree. Dynamic text is re-evaluated by the app logic itself (the plan's
//! "no clones needed for text because views are rebuilt").
//!
//! # Zero-overhead contract
//!
//! Static children are native tuples (`flex(Axis, (a, b, c))`), never
//! `Vec<Box<..>>`. Boxing appears only at dynamic seams (`if`/`else`
//! branches and `for` bodies), where a hand-written Xilem application pays
//! the same cost.
//!
//! # Style without inheritance
//!
//! Masonry has no style inheritance, so text decls (`text-*`, `font-*`)
//! declared on containers are cascaded at compile time into every text leaf
//! (the `TextStyleCx` threaded through the walker). Box-model decls wrap the
//! container in a `sized_box`, whose widget carries the `Dimensions`,
//! `Background`, `BorderWidth`, `BorderColor` and `CornerRadius` properties.

use crate::emit::cfg::wrap_with_cfg;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

mod button;
pub(crate) mod component;
mod control_flow;
mod generic;
mod handler;
mod input;
mod style;
mod widgets;

#[must_use]
/// Expands a `quoin_render!` AST into a Xilem `WidgetView` expression.
pub fn emit_render(node: &RenderNode) -> TokenStream {
    let mut ctx = EmitterCtx::new();
    let inner = emit_node(node, &mut ctx, false);

    let mut hoists = ctx
        .captured
        .iter()
        .map(|id| quote! { let #id = ::core::clone::Clone::clone(&#id); })
        .collect::<Vec<_>>();
    // The builder chains emitted below (`.color`, `.padding`, `.gap`, …)
    // are `xilem::style::Style` trait methods; the trait must be in scope
    // even in crates that never import it themselves.
    hoists.push(quote! { use ::xilem::style::Style as _; });
    hoists.extend(ctx.bindings);

    let tokens = if hoists.is_empty() {
        quote! { { #inner } }
    } else {
        quote! { { #(#hoists)* #inner } }
    };

    wrap_with_cfg(node.attrs(), tokens)
}

/// Emission state threaded through the walker: hoisted `let` bindings, the
/// outer variables every closure captures, loop patterns to skip, and the
/// compile-time cascaded text style (Masonry has no inheritance).
pub(crate) struct EmitterCtx {
    /// `let` statements hoisted ahead of the view expression.
    pub bindings: Vec<TokenStream>,
    /// Outer variable idents captured somewhere in the tree.
    pub captured: Vec<proc_macro2::Ident>,
    /// Loop patterns (and other locals introduced by the macro itself) that
    /// the hoist must skip.
    pub excluded: std::collections::HashSet<String>,
    /// Compile-time cascaded text style for the current subtree.
    pub text: TextStyleCx,
}

impl EmitterCtx {
    pub(crate) fn new() -> Self {
        Self {
            bindings: Vec::new(),
            captured: Vec::new(),
            excluded: std::collections::HashSet::new(),
            text: TextStyleCx::default(),
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

/// The cascaded text style: Xilem text properties live on the leaf widgets,
/// so container decls are threaded down and applied to every label.
#[derive(Clone, Default)]
pub(crate) struct TextStyleCx {
    /// `color(...)` token (a `::xilem::Color` expression).
    pub color: Option<TokenStream>,
    /// `text_size(f32)` value.
    pub size: Option<f32>,
    /// `weight(FontWeight)` value.
    pub weight: Option<u16>,
    /// `text_alignment(...)` token.
    pub alignment: Option<TokenStream>,
}

impl TextStyleCx {
    /// The label method chain applying this style.
    ///
    /// `text_size`, `weight` and `text_alignment` are inherent `Label`
    /// methods — they must be applied directly to the label, before any
    /// `Style`-trait builder (`.color`), which wraps the view in a `Prop`
    /// and hides the inherent methods.
    pub(crate) fn label_chain(&self) -> TokenStream {
        let mut chain = quote! {};
        if let Some(size) = self.size {
            chain = quote! { #chain.text_size(#size) };
        }
        if let Some(weight) = self.weight {
            let w = style::weight_const(weight);
            chain = quote! { #chain.weight(#w) };
        }
        if let Some(alignment) = &self.alignment {
            chain = quote! { #chain.text_alignment(#alignment) };
        }
        if let Some(color) = &self.color {
            chain = quote! { #chain.color(#color) };
        }
        chain
    }
}

/// The maximum arity of a flex view-sequence tuple before chunking.
pub(crate) const MAX_TUPLE_ARITY: usize = 12;

pub(crate) fn emit_node(node: &RenderNode, ctx: &mut EmitterCtx, inside_for: bool) -> TokenStream {
    match node {
        RenderNode::Element(el) => emit_element(el, ctx, inside_for),
        RenderNode::Text(t) => text_leaf(quote! { ::xilem::view::label(#t) }, ctx),
        RenderNode::Expr(e) => {
            // The expression is evaluated eagerly per rebuild; its captures
            // still need the top-of-pass clones so several consumers can
            // share one signal.
            let idents = crate::transpile::collect_handler_idents_excluding_params(e);
            for ident in &idents {
                ctx.capture(ident);
            }
            text_leaf(quote! { ::xilem::view::label(#e) }, ctx)
        }
        RenderNode::If(if_node) => control_flow::emit_if(if_node, ctx),
        RenderNode::For(for_node) => control_flow::emit_for(for_node, ctx),
        RenderNode::Root(nodes) => {
            let views: Vec<TokenStream> = nodes
                .iter()
                .map(|n| emit_node(n, ctx, inside_for))
                .collect();
            if views.len() == 1 {
                views[0].clone()
            } else {
                generic::flex_sequence(&views, quoin_style::FlexDir::Column)
            }
        }
    }
}

/// A text leaf: applies the cascaded text style to the label.
fn text_leaf(label: TokenStream, ctx: &EmitterCtx) -> TokenStream {
    let chain = ctx.text.label_chain();
    if chain.is_empty() {
        label
    } else {
        quote! { #label #chain }
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
        "separator" => widgets::emit_separator(el, ctx),
        "checkbox" => widgets::emit_checkbox(el, ctx),
        "switch" => widgets::emit_switch(el, ctx),
        "slider" => widgets::emit_slider(el, ctx),
        "scroll_area" => widgets::emit_scroll_area(el, ctx, inside_for),
        "virtual_list" => widgets::emit_virtual_list(el, ctx),
        "progress" => widgets::emit_progress(el, ctx),
        "tabs" | "tab_bar" => widgets::emit_tabs(el, ctx, inside_for),
        "data_table" => widgets::emit_data_table(el, ctx),
        "dropdown_menu" => widgets::emit_dropdown_menu(el, ctx),
        "badge" => widgets::emit_badge(el, ctx, inside_for),
        "skeleton" | "skeleton_text" | "skeleton_avatar" => widgets::emit_skeleton(el, ctx),
        "tooltip" => widgets::unsupported_element(
            &name_str,
            "Xilem has no tooltip widget yet; wrap the target in a styled container with a hint label",
        ),
        "rich_text" | "styled_text" => widgets::unsupported_element(
            &name_str,
            "rich text is not on the v1 Xilem parity path; use `text` with styled `span`s",
        ),
        "icon" => {
            widgets::unsupported_element(&name_str, "inline icon SVGs are a GPUI-backend feature")
        }
        "img" | "svg" => {
            widgets::unsupported_element(&name_str, "images are not on the v1 Xilem parity path")
        }
        "clipboard_button" => {
            widgets::unsupported_element(&name_str, "clipboard access lands with the M5 Xilem push")
        }
        "radio_group" | "radio" => widgets::unsupported_element(
            &name_str,
            "radio inputs land with the M5 widget push; use `checkbox` meanwhile",
        ),
        _ => generic::emit_container(el, ctx, inside_for),
    }
}
