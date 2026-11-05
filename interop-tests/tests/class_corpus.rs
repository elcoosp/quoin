//! Phase-0 CI gate #2 — the class corpus.
//!
//! The "CI signal before any change" deliverable of the migration plan (§8
//! Phase 0) and the concrete mitigation for its risk 3: *the class set
//! accepted on GPUI must remain a superset of the set accepted on web*, so
//! the engine swap cannot break a cross-backend component.
//!
//! APIs: `quoin_style::KNOWN_CLASSES` (static slice), `gpui_rsx_core::
//! known_classes()`, `SUPPORTED_VARIANTS`, `analyze_class` + `ClassMode` /
//! `SpacingScale` / `ClassOutcome` — all compiled against the real crates on
//! 2026-11-04 (see `src/bin/class_engine_probe.rs`, the probe this grew from).

use std::sync::Arc;

use gpui_rsx_core::{
    ClassMode, ClassOutcome, SpacingScale, SUPPORTED_VARIANTS, analyze_class, known_classes,
};
use quoin_style::KNOWN_CLASSES;

/// Token names quoin resolves itself (`bg-`/`text-` prefixed); these are
/// legitimate even when absent from gpui-rsx's *static* known-class list —
/// they resolve through the theme (`tests/theme_parity.rs` covers them).
const QUOIN_TOKEN_NAMES: &[&str] = &[
    "primary",
    "secondary",
    "background",
    "foreground",
    "muted",
    "muted-foreground",
    "accent",
    "info",
    "warning",
    "danger",
    "destructive",
    "success",
    "border",
    "input",
    "ring",
    "card",
    "popover",
];

fn is_theme_token_class(class: &str) -> bool {
    class
        .strip_prefix("bg-")
        .or_else(|| class.strip_prefix("text-"))
        .is_some_and(|rest| QUOIN_TOKEN_NAMES.contains(&rest))
}

// ── Vocabulary conformance (risk-3 mitigation) ──────────────────────────
/// Every quoin-known class must be *resolvable* through the engine that
/// would take over at the Phase-5 flip: in gpui-rsx's static table, a quoin
/// theme token, or resolvable by `analyze_class` (generic numeric parsing
/// covers spacing/sizing/opacity; identifier-shaped classes pass through as
/// GPUI method calls — how bare `rounded` / `font-mono` lower). A class
/// landing in `Error`/`Ignored` would break a cross-backend component at the
/// flip — that is the gap this gate prints.
#[test]
fn gpui_rsx_resolves_every_quoin_class() {
    let grsx: std::collections::HashSet<&str> =
        known_classes().iter().map(|s| s.as_str()).collect();
    let tw_mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);

    let mut broken: Vec<&str> = Vec::new();
    for class in KNOWN_CLASSES {
        if grsx.contains(class) || is_theme_token_class(class) {
            continue;
        }
        match analyze_class(class, tw_mode.clone()).outcome {
            ClassOutcome::Supported(_) | ClassOutcome::PassThrough { .. } => {}
            ClassOutcome::Ignored { message } | ClassOutcome::Error { message } => {
                broken.push(class);
                eprintln!("`{class}`: {message}");
            }
            _ => broken.push(class),
        }
    }

    // Deliberate exceptions (migration report §2.1/§4.2, risk 4): the three
    // classes emitting methods absent from gpui-pre 0.3.10 — broken in both
    // engines today (latent upstream bug, reported to both projects).
    let known_broken = ["inline-flex", "inline-block", "select-none"];
    let unexpected: Vec<&str> = broken
        .iter()
        .copied()
        .filter(|c| !known_broken.contains(c))
        .collect();
    assert!(
        unexpected.is_empty(),
        "quoin classes the engine cannot resolve — triage each (generic \
         numeric/passthrough resolution is fine; Error/Ignored is a real gap):\
         \n  {unexpected:#?}"
    );
}

#[test]
fn vocabulary_sizes_stay_in_documented_range() {
    // Report §4: quoin KNOWN_CLASSES = 327, gpui-rsx = 1,158 + generic
    // numeric parsing. Wide bounds on purpose — they catch accidental
    // truncation of either table (e.g. a bad merge), not drift.
    assert!(
        (300..=400).contains(&KNOWN_CLASSES.len()),
        "quoin KNOWN_CLASSES = {} — documented as 327",
        KNOWN_CLASSES.len()
    );
    assert!(
        known_classes().len() >= 1_000,
        "gpui-rsx known_classes() = {} — documented as 1,158",
        known_classes().len()
    );
}

// ── State variants: quoin needs `hover:` today; the swap unlocks the rest ───
#[test]
fn state_variants_cover_quoins_needs() {
    for variant in ["hover", "focus", "active"] {
        assert!(
            SUPPORTED_VARIANTS.contains(&variant),
            "variant `{variant}` missing from gpui-rsx SUPPORTED_VARIANTS = {SUPPORTED_VARIANTS:?}"
        );
    }
}

/// Expands one class under an explicit mode; `None` unless Supported.
fn expand(class: &str, mode: ClassMode) -> Option<String> {
    match analyze_class(class, mode).outcome {
        ClassOutcome::Supported(ts) => Some(format!("{ts}")),
        _ => None,
    }
}

// ── Spacing scale (risk 1 — "the gotcha that bites") ────────────────────────
#[test]
fn tailwind_spacing_scale_is_active_and_distinct_from_default() {
    let px_mode = ClassMode::PERMISSIVE;
    let tw_mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);

    // quoin contract: `p-4` = 16 px (4 px per unit — `Len::spacing`).
    let tw = expand("p-4", tw_mode)
        .unwrap_or_else(|| panic!("`p-4` stopped being Supported under the Tailwind scale"));
    let px = expand("p-4", px_mode).expect("`p-4` stopped being Supported under the default scale");

    assert_ne!(
        tw, px,
        "SpacingScale::Tailwind stopped changing the expansion — scale option lost"
    );
    assert!(
        tw.contains("16"),
        "Tailwind-scale `p-4` no longer expands to a 16 px value: `{tw}`\n\
         (quoin contract: 1 unit = 4 px — if this fires after a gpui-rsx bump, \
         re-check SpacingScale::Tailwind)"
    );
    assert!(
        px.contains("4") && !px.contains("16"),
        "default-scale `p-4` unexpectedly carries a 16: `{px}`"
    );
}

// ── The report's demo string must resolve on the GPUI path ─────────────
#[test]
fn multi_class_demo_string_resolves_under_the_engine() {
    // From the migration report §4.1/§4.2: `hover:text-blue-400` did not
    // compile on quoin's engine; under the flag every class of this string
    // resolves through the same decision the engine bridge makes
    // (quoin-known → quoin lowering; quoin-unknown → gpui-rsx).
    let full = "flex flex-col gap-4 p-4 bg-gray-900 text-white hover:text-blue-400";

    let tw_mode = ClassMode::PERMISSIVE
        .with_scale(SpacingScale::Tailwind)
        .with_theme(Some(Arc::new(
            gpui_rsx_core::ThemeBuilder::default()
                .scale(SpacingScale::Tailwind)
                .build(),
        )));
    for class in full.split_whitespace() {
        let quoin_parsed = quoin_style::parse_tailwind(class);
        let outcome = analyze_class(class, tw_mode.clone()).outcome;
        let engine_resolves = matches!(outcome, ClassOutcome::Supported(_));
        assert!(
            quoin_parsed.unknown.is_empty() || engine_resolves,
            "`{class}` resolves on neither engine — a cross-backend component \
             carrying it would break"
        );
    }
    // The documented gap itself: text-blue-400 is exactly the class quoin
    // rejects and the engine resolves (report §4.2).
    let parsed = quoin_style::parse_tailwind("text-blue-400");
    assert!(!parsed.unknown.is_empty(), "text-blue-400 should be outside quoin's palette");
    assert!(
        matches!(
            analyze_class("text-blue-400", tw_mode).outcome,
            ClassOutcome::Supported(_)
        ),
        "text-blue-400 must resolve through the engine"
    );
}

// ── Classes the report lists as newly gained must resolve ───────────────────
#[test]
fn previously_unknown_classes_resolve_through_gpui_rsx() {
    let tw_mode = ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind);
    for class in [
        "text-blue-400",
        "border-t-2",
        "focus:border-blue-500",
        "active:bg-blue-700",
        "group-hover:bg-red-500",
        "w-1/2",
    ] {
        let outcome = analyze_class(class, tw_mode.clone()).outcome;
        assert!(
            matches!(outcome, ClassOutcome::Supported(_)),
            "`{class}` (listed as a vocabulary gain in report §4.2) is not Supported"
        );
    }
    // Negative utilities are NOT in gpui-rsx-core 0.10.2 (upstream main added
    // them later, commit #108): not a regression — quoin rejects them too.
    assert!(
        !matches!(
            analyze_class("-mt-2", tw_mode).outcome,
            ClassOutcome::Supported(_)
        ),
        "-mt-2 unexpectedly resolved on 0.10.2 — update the vocabulary notes"
    );
}
