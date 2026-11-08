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
//! behavioral elements (`dialog`, `popover`, `command`, …) land with their own
//! contract rows once their overlay seams exist on the GPU backends. Phase 2
//! added the first Tier-2 rows — the disclosure family (`accordion*`) plus
//! the canonical `tabs`/`tooltip` upgrades — none of which need an overlay
//! seam: they are inline, signal-driven behavior.
//!
//! Tier-1 elements are lowered **generically** (styled containers/text) on
//! both dynamic backends when no shadcn backing is enabled, which keeps them
//! outside the v1 `BACKEND_PARITY` matrix (the "plain HTML-ish containers"
//! category). Enabling a `<framework>-shadcn` feature upgrades the lowering
//! to the real component library; disabling it must keep everything
//! compiling — the cfg-switched emitters enforce that both ways.

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
    /// `shadcn-dioxus` (`MBeliou`) module or fallback note for Dioxus.
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
        gpui: "shim (Phase 2): styled div composition (real Card lands with gpui-kit #2311)",
        xilem: "generic bordered container",
    },
    CanonicalElement {
        name: "card_header",
        floem: "floem_shadcn::components::card::CardHeader",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardHeader part)",
        gpui: "shim (Phase 2): styled div (title/description args)",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_title",
        floem: "floem_shadcn::components::card::CardTitle",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardTitle part)",
        gpui: "shim (Phase 2): styled text",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "card_description",
        floem: "floem_shadcn::components::card::CardDescription",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardDescription part)",
        gpui: "shim (Phase 2): muted styled text",
        xilem: "generic muted text",
    },
    CanonicalElement {
        name: "card_content",
        floem: "floem_shadcn::components::card::CardContent",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardContent part)",
        gpui: "shim (Phase 2): padded container",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_footer",
        floem: "floem_shadcn::components::card::CardFooter",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardFooter part)",
        gpui: "shim (Phase 2): padded footer row",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "card_action",
        floem: "floem_shadcn::components::card::CardAction",
        leptos: "leptos-shadcn-card",
        dioxus: "shadcn_dioxus::card (CardAction part)",
        gpui: "shim (Phase 2): right-aligned action row",
        xilem: "generic container",
    },
    CanonicalElement {
        name: "alert",
        floem: "floem_shadcn::components::alert::Alert",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert",
        gpui: "shim (Phase 2): tinted container + title/description args",
        xilem: "generic text stack",
    },
    CanonicalElement {
        name: "alert_title",
        floem: "generic (Alert takes title:/description: args)",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert (AlertTitle part)",
        gpui: "shim (Phase 2): medium-weight text",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "alert_description",
        floem: "generic (Alert takes title:/description: args)",
        leptos: "leptos-shadcn-alert",
        dioxus: "shadcn_dioxus::alert (AlertDescription part)",
        gpui: "shim (Phase 2): muted text",
        xilem: "generic muted text",
    },
    CanonicalElement {
        name: "spinner",
        floem: "floem_shadcn::components::spinner::Spinner",
        leptos: "leptos-shadcn-spinner",
        dioxus: "shadcn_dioxus::spinner",
        gpui: "gpui_component::spinner::Spinner (real, Phase 2)",
        xilem: "generic static glyph",
    },
    CanonicalElement {
        name: "kbd",
        floem: "floem_shadcn::components::kbd::Kbd",
        leptos: "leptos-shadcn-kbd",
        dioxus: "shadcn_dioxus::kbd",
        gpui: "shim (Phase 2): key-cap chip (real Kbd needs the #2311 text API)",
        xilem: "generic text",
    },
    CanonicalElement {
        name: "avatar",
        floem: "floem_shadcn::components::avatar::Avatar",
        leptos: "leptos-shadcn-avatar",
        dioxus: "shadcn_dioxus::avatar",
        gpui: "shim (Phase 2): initials circle (real Avatar lands with #2311)",
        xilem: "generic fallback text",
    },
    CanonicalElement {
        name: "avatar_fallback",
        floem: "generic (Avatar takes fallback: arg)",
        leptos: "leptos-shadcn-avatar",
        dioxus: "shadcn_dioxus::avatar (AvatarFallback part)",
        gpui: "shim (Phase 2): initials text",
        xilem: "generic text",
    },
];

/// The canonical Tier-2 set (rich-elements Phase 2): inline, signal-driven
/// behavior elements — the disclosure family plus the canonical `tabs` and
/// `tooltip` rows. Same rules as Tier-1: every row is a known element, and
/// every backend must lower it (real component, styled shim, or generic
/// fallback) without new dependency requirements.
pub const CANONICAL_TIER2: &[CanonicalElement] = &[
    CanonicalElement {
        name: "accordion",
        floem: "floem_shadcn::components::accordion::Accordion",
        leptos: "leptos-shadcn-accordion (feature; generic lowering in quoin)",
        dioxus: "shadcn_dioxus::accordion (module exists; quoin lowers generic)",
        gpui: "generic fallback (disclosure shim planned with #2311)",
        xilem: "generic disclosure container",
    },
    CanonicalElement {
        name: "accordion_item",
        floem: "floem_shadcn::components::accordion_rich::RichAccordionItem",
        leptos: "leptos-shadcn-accordion (AccordionItem)",
        dioxus: "shadcn_dioxus::accordion (AccordionItem)",
        gpui: "generic fallback (rows keyed by `value:`)",
        xilem: "generic rows",
    },
    CanonicalElement {
        name: "accordion_trigger",
        floem: "RichAccordionItem trigger (any view, chevron + click toggle)",
        leptos: "leptos-shadcn-accordion (AccordionTrigger)",
        dioxus: "shadcn_dioxus::accordion (AccordionTrigger)",
        gpui: "generic fallback",
        xilem: "generic row",
    },
    CanonicalElement {
        name: "accordion_content",
        floem: "RichAccordionItem content (any view, Display::None collapsed)",
        leptos: "leptos-shadcn-accordion (AccordionContent)",
        dioxus: "shadcn_dioxus::accordion (AccordionContent)",
        gpui: "generic fallback",
        xilem: "generic region",
    },
    CanonicalElement {
        name: "tabs",
        floem: "floem_shadcn::components::tabs::Tabs (Phase 2 upgrade; usize bridge)",
        leptos: "leptos-shadcn-tabs (feature; pre-existing lowering)",
        dioxus: "generic strip + keyed content (pre-existing)",
        gpui: "gpui-component tabs (pre-existing lowering)",
        xilem: "generic strip + keyed content",
    },
    CanonicalElement {
        name: "tooltip",
        floem: "floem_shadcn::components::tooltip::TooltipExt (Phase 2 upgrade)",
        leptos: "leptos-shadcn-tooltip (feature; pre-existing lowering)",
        dioxus: "generic title attr (pre-existing)",
        gpui: "gpui-component tooltip (pre-existing)",
        xilem: "generic (hover story pending upstream)",
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

// The choice readers consume the shared argument helpers (`emit::common`,
// which only exists on framework builds), so the module needs one base
// feature AND the shadcn flavor that actually consumes it. The GPUI shims
// are unconditional on `gpui`; the other backends call it under their
// `-shadcn` features.
#[cfg(any(
    feature = "gpui",
    all(feature = "leptos", feature = "leptos-shadcn"),
    all(feature = "dioxus", feature = "dioxus-shadcn"),
    all(feature = "floem", feature = "floem-shadcn")
))]
pub(crate) mod choice {
    use super::BUTTON_VARIANTS;
    use crate::emit::common::{find_arg_bool, find_arg_string};
    use crate::render_ast::Element;

    /// Turns a `canonical_choice` result into the emitted diagnostic for an
    /// unknown value; the `Ok` branch is the caller's mapping job.
    pub(crate) fn choice_error(
        value: &Result<&'static str, String>,
    ) -> Option<proc_macro2::TokenStream> {
        match value {
            Ok(_) => None,
            Err(message) => Some(quote::quote! { ::core::compile_error!(#message) }),
        }
    }

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
    pub(crate) fn canonical_button_variant(el: &Element) -> Option<Result<&'static str, String>> {
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
    /// Consumed by the GPUI button shim and the Floem shadcn lowering.
    #[cfg(any(feature = "gpui", all(feature = "floem", feature = "floem-shadcn")))]
    pub(crate) fn canonical_button_size(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(el, "size", super::BUTTON_SIZES, &[])
    }

    /// The canonical `badge` variant (or `Err` for an unknown value).
    /// Only the Floem shadcn lowering picks badge variants today.
    #[cfg(all(feature = "floem", feature = "floem-shadcn"))]
    pub(crate) fn canonical_badge_variant(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(
            el,
            "variant",
            super::BADGE_VARIANTS,
            &[
                ("secondary", "secondary"),
                ("destructive", "destructive"),
                ("outline", "outline"),
            ],
        )
    }

    /// The canonical `alert` variant (or `Err` for an unknown value).
    /// Consumed by the GPUI alert shim and the Floem shadcn lowering.
    #[cfg(any(feature = "gpui", all(feature = "floem", feature = "floem-shadcn")))]
    pub(crate) fn canonical_alert_variant(el: &Element) -> Option<Result<&'static str, String>> {
        canonical_choice(
            el,
            "variant",
            super::ALERT_VARIANTS,
            &[("destructive", "destructive")],
        )
    }
}

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

    /// Every canonical Tier-2 row must also be an accepted `quoin_render!`
    /// element — the contract table and the parser cannot drift.
    #[test]
    fn tier2_rows_are_known_elements() {
        for row in CANONICAL_TIER2 {
            assert!(
                crate::render_ast::known_elements().contains(&row.name),
                "canonical Tier-2 element `{}` is not in KNOWN_ELEMENTS",
                row.name
            );
        }
    }

    /// Tier-2 row names must be unique *within their tier and across tiers*
    /// — an element belongs to exactly one contract tier.
    #[test]
    fn tier2_rows_are_unique_and_disjoint_from_tier1() {
        let mut names: Vec<_> = CANONICAL_TIER2.iter().map(|r| r.name).collect();
        let tier1: Vec<_> = CANONICAL_TIER1.iter().map(|r| r.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), CANONICAL_TIER2.len(), "duplicate Tier-2 rows");
        for name in &names {
            assert!(
                !tier1.contains(name),
                "element `{name}` appears in both Tier-1 and Tier-2"
            );
        }
    }

    /// The choice readers (and the emitters that consume them) are compiled
    /// on every framework build with a shadcn lowering; exercise the grammar
    /// here so the "string arg wins, sugar breaks ties" rule cannot regress.
    /// The Floem shadcn build is the superset consumer (all four readers).
    #[cfg(all(feature = "floem", feature = "floem-shadcn"))]
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
            assert_eq!(
                canonical_button_variant(&el).unwrap().unwrap(),
                "destructive"
            );
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
            assert!(super::choice::choice_error(&canonical_button_variant(&el).unwrap()).is_some());
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
