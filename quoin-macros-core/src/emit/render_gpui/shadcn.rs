//! GPUI lowerings for the canonical shadcn surface — rich-elements Phase 2,
//! the "shimmable parts" of gpui-kit discussion #2311.
//!
//! gpui-component 0.8.1 does not (yet) ship every shadcn surface component
//! under canonical names (#2311 asks upstream for `Card`, canonical
//! `destructive` naming, and friends). Until those land, this module lowers
//! the Tier-1 surface elements as **styled shims**: plain GPUI divs carrying
//! the shadcn look through the *same* class transpiler a user's `class:`
//! argument goes through, so shim chrome and user chrome stay consistent and
//! theme-token-driven (`bg-card`, `text-muted-foreground`, …).
//!
//! One element is a real component: `spinner` lowers to
//! `gpui_component::spinner::Spinner` (0.8.1 ships it). `button`, `input`,
//! `tabs`, `tooltip`, and `dropdown_menu` already had real lowerings before
//! Phase 2 and are dispatched elsewhere.
//!
//! Everything here flows through
//! [`super::generic::finish_with_class_children_handlers`], so user
//! `class:` args layer on top of the shim chrome and children/handlers
//! behave exactly like the generic lowering.

use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::generic::finish_with_class_children_handlers;

/// Whether `name` is one of the shimmed surface elements.
pub(crate) fn is_shim_element(name: &str) -> bool {
    matches!(
        name,
        "card"
            | "card_header"
            | "card_title"
            | "card_description"
            | "card_content"
            | "card_footer"
            | "card_action"
            | "alert"
            | "alert_title"
            | "alert_description"
            | "spinner"
            | "kbd"
            | "avatar"
            | "avatar_fallback"
    )
}

/// Applies one shim chrome class string through the active class engine.
fn shim_chrome(chain: TokenStream, shim_class: &str) -> TokenStream {
    let styles = {
        #[cfg(feature = "gpui-rsx-classes")]
        {
            crate::transpile::tailwind_grsx::transpile_class(shim_class)
        }
        #[cfg(not(feature = "gpui-rsx-classes"))]
        {
            crate::transpile::tailwind::transpile_class(shim_class)
        }
    };
    crate::transpile::tailwind::apply_normal(chain, &styles)
}

/// The dispatcher for every shim element. Callers check
/// [`is_shim_element`] first.
pub(crate) fn emit_shim_element(el: &Element) -> TokenStream {
    match el.name.to_string().as_str() {
        "card" => emit_card(el),
        "card_header" => emit_card_header(el),
        "card_title" => emit_card_title(el),
        "card_description" => emit_card_description(el),
        "card_content" => emit_card_content(el),
        "card_footer" => emit_card_footer(el),
        "card_action" => emit_card_action(el),
        "alert" => emit_alert(el),
        "alert_title" => emit_alert_title(el),
        "alert_description" => emit_alert_description(el),
        "spinner" => emit_spinner(el),
        "kbd" => emit_kbd(el),
        "avatar" => emit_avatar(el),
        "avatar_fallback" => emit_avatar_fallback(el),
        // `is_shim_element` guards the dispatcher; this arm is unreachable.
        _ => quote! { ::gpui::div() },
    }
}

// ── card family ─────────────────────────────────────────────────────────────

/// `card` — shadcn surface panel: `rounded-lg border bg-card p-6`.
pub(crate) fn emit_card(el: &Element) -> TokenStream {
    let chain = shim_chrome(
        quote! { ::gpui::div() },
        "flex flex-col rounded-lg border bg-card p-6 gap-2 shadow-xs text-foreground",
    );
    finish_with_class_children_handlers(el, chain)
}

/// `card_header` — title block above the card body.
pub(crate) fn emit_card_header(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "flex flex-col gap-2");
    finish_with_class_children_handlers(el, chain)
}

/// `card_title` — prominent card heading.
pub(crate) fn emit_card_title(el: &Element) -> TokenStream {
    let chain = shim_chrome(
        quote! { ::gpui::div() },
        "text-lg font-semibold text-foreground",
    );
    finish_with_class_children_handlers(el, chain)
}

/// `card_description` — muted supporting text under the title.
pub(crate) fn emit_card_description(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "text-sm text-muted-foreground");
    finish_with_class_children_handlers(el, chain)
}

/// `card_content` — the card body region.
pub(crate) fn emit_card_content(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "flex flex-col gap-4");
    finish_with_class_children_handlers(el, chain)
}

/// `card_footer` — action row at the card bottom.
pub(crate) fn emit_card_footer(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "flex items-center gap-2");
    finish_with_class_children_handlers(el, chain)
}

/// `card_action` — right-aligned action cluster in the header row.
pub(crate) fn emit_card_action(el: &Element) -> TokenStream {
    let chain = shim_chrome(
        quote! { ::gpui::div() },
        "flex items-center justify-end gap-2",
    );
    finish_with_class_children_handlers(el, chain)
}

// ── alert family ────────────────────────────────────────────────────────────

/// `alert(title:, description:)` — tinted notice panel. The canonical
/// `variant:` grammar applies (`default`/`destructive`); destructive tints
/// the border and text through the theme's destructive token.
pub(crate) fn emit_alert(el: &Element) -> TokenStream {
    let choice = crate::emit::shadcn_spec::canonical_alert_variant(el);
    if let Some(error) = choice
        .as_ref()
        .and_then(crate::emit::shadcn_spec::choice_error)
    {
        return error;
    }
    let chrome = match choice.and_then(|c| c.ok()) {
        Some("destructive") => {
            "flex flex-col gap-1 rounded-lg border border-destructive bg-card p-4 text-destructive"
        }
        _ => "flex flex-col gap-1 rounded-lg border bg-card p-4 text-foreground",
    };
    let mut chain = shim_chrome(quote! { ::gpui::div() }, chrome);

    // The shared Floem contract carries the copy as `title:`/`description:`
    // args; render them as the alert's own title/description rows.
    if let Some(title) = find_arg_string(el, "title") {
        chain = quote! {
            #chain.child(::gpui::div().text_sm().font_weight(::gpui::FontWeight::MEDIUM).child(#title))
        };
    }
    if let Some(description) = find_arg_string(el, "description") {
        chain = quote! {
            #chain.child(::gpui::div().text_sm().opacity(0.9).child(#description))
        };
    }

    finish_with_class_children_handlers(el, chain)
}

/// `alert_title` — the alert's medium-weight heading row.
pub(crate) fn emit_alert_title(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "text-sm font-medium");
    finish_with_class_children_handlers(el, chain)
}

/// `alert_description` — the alert's muted body text.
pub(crate) fn emit_alert_description(el: &Element) -> TokenStream {
    let chain = shim_chrome(quote! { ::gpui::div() }, "text-sm text-muted-foreground");
    finish_with_class_children_handlers(el, chain)
}

// ── spinner / kbd / avatar ──────────────────────────────────────────────────

/// `spinner` — a real `gpui_component::spinner::Spinner` (0.8.1 ships it),
/// wrapped so `class:` styles still land on a plain div.
pub(crate) fn emit_spinner(el: &Element) -> TokenStream {
    let chain = quote! { ::gpui::div().child(::gpui_component::spinner::Spinner::new()) };
    finish_with_class_children_handlers(el, chain)
}

/// `kbd` — key-cap chip. Text source mirrors the Floem contract exactly:
/// `text:` / `label:` argument, or exactly one text child.
pub(crate) fn emit_kbd(el: &Element) -> TokenStream {
    let text = match find_arg_string(el, "text").or_else(|| find_arg_string(el, "label")) {
        Some(text) => Some(text),
        None => {
            let mut texts = Vec::new();
            for child in &el.children {
                if let crate::render_ast::RenderNode::Text(lit) = child {
                    texts.push(lit.clone());
                }
            }
            match texts.as_slice() {
                [lit] => Some(lit.value()),
                _ => None,
            }
        }
    };
    let Some(text) = text else {
        return quote! {
            ::core::compile_error!("`kbd` requires `text:` (or exactly one text child)")
        };
    };
    let chain = shim_chrome(
        quote! { ::gpui::div() },
        "flex items-center justify-center rounded border bg-muted px-1p5 py-1 font-mono text-xs font-medium text-foreground",
    );
    let chain = quote! { #chain.child(#text) };
    finish_with_class_children_handlers(el, chain)
}

/// `avatar(fallback:, size:)` — initials circle. The size argument is a
/// raw pixel length (fixed sizes are not in the class vocabulary); the
/// circle chrome is class-driven.
pub(crate) fn emit_avatar(el: &Element) -> TokenStream {
    let size = find_arg_f32(el, "size").unwrap_or(32.0);
    let size_lit = proc_macro2::Literal::f32_suffixed(size);
    let mut chain = shim_chrome(
        quote! { ::gpui::div() },
        "flex items-center justify-center overflow-hidden rounded-full bg-muted text-xs font-medium text-foreground shrink-0",
    );
    chain = quote! { #chain.w(::gpui::px(#size_lit)).h(::gpui::px(#size_lit)) };

    // Initials: the `fallback:` argument, else the first text child.
    let initials = match find_arg_string(el, "fallback") {
        Some(fallback) => Some(fallback),
        None => {
            let mut texts = Vec::new();
            for child in &el.children {
                if let crate::render_ast::RenderNode::Text(lit) = child {
                    texts.push(lit.clone());
                }
            }
            match texts.as_slice() {
                [lit] => Some(lit.value()),
                _ => None,
            }
        }
    };
    if let Some(initials) = initials {
        chain = quote! { #chain.child(#initials) };
    }

    finish_with_class_children_handlers(el, chain)
}

/// `avatar_fallback` — the muted initials text (usually nested in `avatar`).
pub(crate) fn emit_avatar_fallback(el: &Element) -> TokenStream {
    let chain = shim_chrome(
        quote! { ::gpui::div() },
        "text-xs font-medium text-muted-foreground",
    );
    finish_with_class_children_handlers(el, chain)
}
