//! The canonical shadcn element contract — rich-elements Phase 0.
//!
//! This module is the checked-in, machine-readable half of the
//! shadcn rich-elements plan (`docs/quoin-shadcn-rich-elements-evaluation.md`
//! §5). It pins down, in code:
//!
//! 1. **The canonical Tier-1 element set** — the shadcn/ui surface elements
//!    quoin accepts on every backend, each with the per-framework backing
//!    library that realizes it (`floem-shadcn`, `leptos-shadcn-ui`,
//!    `shadcn-dioxus`, gpui-component, or the generic lowering).
//! 2. **The canonical argument grammar** — one spelling per concept:
//!    `variant:` / `size:` string arguments (`variant: "destructive"`),
//!    with the historical boolean sugar (`destructive: true`, `ghost: true`,
//!    …) kept as a fallback so existing sources keep compiling. The
//!    helpers below implement "string arg wins, sugar breaks ties" once,
//!    and every backend emitter routes through them, so the five lowerings
//!    cannot drift.
//!
//! The Tier-1 set is deliberately *presentational / composite-surface* only:
//! everything here lowers without portals, overlays or focus traps. Tier-2
//! behavioral elements (`dialog`, `accordion`, `popover`, `command`, …) land
//! with their own contract rows once their overlay seams exist on the GPU
//! backends.
//!
//! Tier-1 elements are lowered **generically** (styled containers/text) on
//! both dynamic backends when no shadcn backing is enabled, which keeps them
//! outside the v1 `BACKEND_PARITY` matrix (the "plain HTML-ish containers"
//! category). Enabling a `<framework>-shadcn` feature upgrades the lowering
//! to the real component library; disabling it must keep everything
//! compiling — the cfg-switched emitters enforce that both ways.

use proc_macro2::TokenStream;

/// How a canonical element is realized per framework. Purely documentary
/// today (the parity matrix owns the machine-checked dynamic-backend
/// contract); the strings name the exact backing module/crate so the table
/// doubles as the integration map for the roadmap phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalElement {
    /// The element name as written in `quoin_render!`.
    pub name: &'static str,
    /// `floem-shadcn` module backing the Floem lowering.
    pub floem: &'static str,
    /// `leptos-shadcn-ui` component-crate feature backing the Leptos lowering.
    pub leptos: &'static str,
    /// `shadcn-dioxus` (MBeliou) module or fallback note for Dioxus.
    pub dioxus: &'static str,
    /// gpui-component backing (or the shim note until upstream lands it).
    pub gpui: &'static str,
    /// The generic lowering both dynamic backends fall back to.
    pub xilem: &'static str,
}

/// The canonical Tier-1 set: surface & composite elements with shadcn
/// backing available (or shimmed) on every framework backend.
///
/// Keep in sync with `KNOWN_ELEMENTS` — the unit test at the bottom of this
/// module asserts every row is a known element.
pub const CANONICAL_TIER1: &[CanonicalElement] = &[
    CanonicalElement {
        name: "card",
        floem: "floem_shadcn::components::card::Card",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card",
        gpui: "shim: styled div composition (gpui-kit #2311 asks for Card)",
        xilem: "generic bordered container",
    },
    CanonicalElement {
        name: "card_header",
        floem: "floem_shadcn::components::card::CardHeader",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardHeader part)",
        gpui: "shim: styled div (title/description args)",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_title",
        floem: "floem_shadcn::components::card::CardTitle",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardTitle part)",
        gpui: "shim: styled text",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "card_description",
        floem: "floem_shadcn::components::card::CardDescription",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardDescription part)",
        gpui: "shim: muted styled text",
        xilem: "generic muted text",
    },
    CanonicalElement {
        name: "card_content",
        floem: "floem_shadcn::components::card::CardContent",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardContent part)",
        gpui: "shim: padded container",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_footer",
        floem: "floem_shadcn::components::card::CardFooter",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardFooter part)",
        gpui: "shim: padded row container",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_action",
        floem: "floem_shadcn::components::card::CardAction",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardAction part)",
        gpui: "shim: right-aligned row",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "alert",
        floem: "floem_shadcn::components::alert::Alert",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert",
        gpui: "shim: tinted container + title/description",
        xilem: "generic text stack",
    },
    CanonicalElement {
        name: "alert_title",
        floem: "generic (Alert takes title:/description: args)",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert (AlertTitle part)",
        gpui: "shim: medium-weight text",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "alert_description",
        floem: "generic (Alert takes title:/description: args)",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert (AlertDescription part)",
        gpui: "shim: muted text",
        xilem: "generic muted text",
    },
    CanonicalElement {
        name: "spinner",
        floem: "floem_shadcn::components::spinner::Spinner",
        leptos: "leptos-shadcn-spinner",
        dioxus: "shadcn_dioxus::spinner",
        gpui: "shim: animated glyph",
        xilem: "generic static glyph",
    },
    CanonicalElement {
        name: "kbd",
        floem: "floem_shadcn::components::kbd::Kbd",
        leptos: "leptos-shadcn-kbd",
        dioxus: "shadcn_dioxus::kbd",
        gpui: "shim: key-cap styled text",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "avatar",
        floem: "floem_shadcn::components::avatar::Avatar",
        leptos: "leptos-shadcn-avatar",
        dioxus: "shadcn_dioxus::avatar",
        gpui: "shim: initials circle (gpui-kit #2311 asks for Avatar)",
        xilem: "generic fallback text",
    },
    CanonicalElement {
        name: "avatar_fallback",
        floem: "generic (Avatar takes fallback: arg)",
        leptos: "leptos-shadcn-avatar",
        dioxus: "shadcn_dioxus::avatar (AvatarFallback part)",
        gpui: "shim: initials text",
        xilem: "generic text",
    },
];

/// The canonical `button` variant set (shadcn/ui canon, plus `primary` as
/// the accepted alias of `default` for gpui-component parity).
pub const BUTTON_VARIANTS: &[&str] = &[
    "default",
    "primary",
    "destructive",
    "outline",
    "secondary",
    "ghost",
    "link",
];

/// The canonical `button` size set.
pub const BUTTON_SIZES: &[&str] = &["sm", "default", "lg", "icon"];

/// The canonical `badge` variant set.
pub const BADGE_VARIANTS: &[&str] = &["default", "secondary", "destructive", "outline"];

/// The canonical `alert` variant set.
pub const ALERT_VARIANTS: &[&str] = &["default", "destructive"];

/// Turns a `canonical_choice` result into the emitted diagnostic for an
/// unknown value; the `Ok` branch is the caller's mapping job.
///
/// `allow(dead_code)`: the readers are consumed by framework emitters whose
/// cargo features may all be off (e.g. the floem+xilem parity build) — that
/// combination must stay warning-clean.
#[allow(dead_code)]
pub(crate) fn choice_error(value: &Result<&'static str, String>) -> Option<TokenStream> {
    match value {
        Ok(_) => None,
        Err(message) => Some(quote::quote! { ::core::compile_error!(#message) }),
    }
}

// The choice readers consume the shared argument helpers, which only exist
// on framework builds — same gate as `emit::common`.
#[cfg(any(
    feature = "gpui",
    feature = "leptos",
    feature = "dioxus",
    feature = "floem",
    feature = "xilem"
))]
#[allow(dead_code)]
pub(crate) mod choice {
    use super::{ALERT_VARIANTS, BADGE_VARIANTS, BUTTON_SIZES, BUTTON_VARIANTS};
    use crate::emit::common::{find_arg_bool, find_arg_string};
    use crate::render_ast::Element;

    /// Reads the canonical `variant:`/`size:` choice off an element.
    ///
    /// Precedence: the `key` string argument wins; when it is absent, the
    /// `sugar` boolean arguments are scanned in canonical order (first hit
    /// wins, matching the historical per-emitter `if` chains). Returns:
    ///
    /// - `Some(Ok(name))` — a known canonical choice;
    /// - `Some(Err(message))` — the argument was present but not a known
    ///   choice; emitters turn this into a precise `compile_error!`
    ///   (silence is never an option);
    /// - `None` — nothing specified; the emitter applies its backend
    ///   default.
    pub(crate) fn canonical_choice(
        el: &Element,
        key: &str,
        known: &'static [&'static str],
        sugar: &[(&str, &str)],
    ) -> Option<Result<&'static str, String>> {
        if let Some(raw) = find_arg_string(el, key) {
            return match known.iter().find(|choice| **choice == raw) {
                Some(choice) => Some(Ok(choice)),
                None => Some(Err(format!(
                    "unknown `{key}: \"{raw}\"` — expected one of: {}",
                    known.join(", ")
                ))),
            };
        }
        sugar
            .iter()
            .find(|(flag, _)| find_arg_bool(el, flag))
            .and_then(|(_, canonical)| known.iter().find(|c| c == &canonical))
            .copied()
            .map(Ok)
    }

    /// The canonical `button` variant (or `Err` for an unknown value).
    pub(crate) fn canonical_button_variant(
        el: &Element,
    ) -> Option<Result<&'static str, String>> {
        canonical_choice(
            el,
            "variant",
            BUTTON_VARIANTS,
            &[
                ("primary", "primary"),
                ("destructive", "destructive"),
                ("ghost", "ghost"),
                ("outline", "outline"),
                ("secondary", "secondary"),
                ("link", "link"),
            ],
        )
    }

    /// The canonical `button` size (or `Err` for an unknown value).
    pub(crate) fn canonical_button_size(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(el, "size", BUTTON_SIZES, &[])
    }

    /// The canonical `badge` variant (or `Err` for an unknown value).
    pub(crate) fn canonical_badge_variant(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(
            el,
            "variant",
            BADGE_VARIANTS,
            &[
                ("secondary", "secondary"),
                ("destructive", "destructive"),
                ("outline", "outline"),
            ],
        )
    }

    /// The canonical `alert` variant (or `Err` for an unknown value).
    pub(crate) fn canonical_alert_variant(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(
            el,
            "variant",
            ALERT_VARIANTS,
            &[("destructive", "destructive")],
        )
    }
}

#[cfg(any(
    feature = "gpui",
    feature = "leptos",
    feature = "dioxus",
    feature = "floem",
    feature = "xilem"
))]
#[allow(unused_imports)]
pub(crate) use choice::{
    canonical_alert_variant, canonical_badge_variant, canonical_button_size,
    canonical_button_variant, canonical_choice,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Every canonical Tier-1 row must be an accepted `quoin_render!`
    /// element — the contract table and the parser cannot drift.
    #[test]
    fn tier1_rows_are_known_elements() {
        for row in CANONICAL_TIER1 {
            assert!(
                crate::render_ast::known_elements().contains(&row.name),
                "canonical Tier-1 element `{}` is not in KNOWN_ELEMENTS",
                row.name
            );
        }
    }

    /// No two rows may claim the same element name.
    #[test]
    fn tier1_rows_are_unique() {
        let mut names: Vec<_> = CANONICAL_TIER1.iter().map(|r| r.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), CANONICAL_TIER1.len(), "duplicate Tier-1 rows");
    }

    /// The choice readers (and the emitters that consume them) are compiled
    /// on every framework build; exercise the grammar here so the
    /// "string arg wins, sugar breaks ties" rule cannot regress.
    #[cfg(any(
        feature = "gpui",
        feature = "leptos",
        feature = "dioxus",
        feature = "floem",
        feature = "xilem"
    ))]
    mod choice_grammar {
        use super::choice::*;
        use crate::render_ast::{Element, RenderNode};

        fn element(source: proc_macro2::TokenStream) -> Element {
            match syn::parse2::<RenderNode>(source).expect("parseable element") {
                RenderNode::Element(el) => *el,
                other => panic!("expected an element, got {other:?}"),
            }
        }

        #[test]
        fn string_variant_wins_over_sugar() {
            let el = element(quote::quote! { button(variant: "destructive", ghost: true) { "x" } });
            assert_eq!(canonical_button_variant(&el).unwrap().unwrap(), "destructive");
        }

        #[test]
        fn sugar_falls_back_in_canonical_order() {
            let el = element(quote::quote! { button(ghost: true, destructive: true) { "x" } });
            assert_eq!(
                canonical_button_variant(&el).unwrap().unwrap(),
                "destructive"
            );
        }

        #[test]
        fn unknown_variant_is_a_named_error() {
            let el = element(quote::quote! { button(variant: "sparkly") { "x" } });
            let err = canonical_button_variant(&el).unwrap().unwrap_err();
            assert!(err.contains("sparkly") && err.contains("destructive"));
            assert!(super::choice_error(&canonical_button_variant(&el).unwrap()).is_some());
        }

        #[test]
        fn absent_variant_is_none() {
            let el = element(quote::quote! { button() { "x" } });
            assert!(canonical_button_variant(&el).is_none());
            assert!(canonical_button_size(&el).is_none());
        }

        #[test]
        fn badge_and_alert_variants() {
            let badge = element(quote::quote! { badge(variant: "outline") { "x" } });
            assert_eq!(canonical_badge_variant(&badge).unwrap().unwrap(), "outline");
            let alert = element(quote::quote! { alert(destructive: true, title: "x") });
            assert_eq!(
                canonical_alert_variant(&alert).unwrap().unwrap(),
                "destructive"
            );
        }

        #[test]
        fn button_size_canonical() {
            let el = element(quote::quote! { button(size: "lg") { "x" } });
            assert_eq!(canonical_button_size(&el).unwrap().unwrap(), "lg");
            let bad = element(quote::quote! { button(size: "huge") { "x" } });
            assert!(canonical_button_size(&bad).unwrap().is_err());
        }

        #[test]
        fn helpers_never_panic_on_plain_elements() {
            let el = element(syn::parse_quote! { button() { "x" } });
            let _ = canonical_button_variant(&el);
            let _ = canonical_button_size(&el);
            let _ = canonical_badge_variant(&el);
            let _ = canonical_alert_variant(&el);
        }
    }
}
