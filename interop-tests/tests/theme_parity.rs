//! Phase-0 CI gate #1 — theme parity between quoin's baked GPUI palette and
//! gpui-rsx-core's `Theme` machinery.
//!
//! Three independent assertions:
//!
//! 1. quoin's parser still resolves every token class to the frozen
//!    shadcn-dark hex recorded here (drift guard on `Token::surface_hex()`
//!    and `Token::text_hex()` — if the palette changes, this fails and the
//!    baked-hex table consumed by the GPUI lowering must be regenerated).
//! 2. A gpui-rsx `Theme` carrying those same hexes expands every class to
//!    exactly the same RGB (machinery guard; verified 34/34 on 2026-11-04 —
//!    baseline preserved in `theme_parity_baseline.txt`).
//! 3. The `gpui-component` preset expansion shape (`cx.theme().colors.*`) is
//!    stable, and `card` remains the one quoin token name without a preset
//!    field.
//!
//! Every API used here was compiled against the real crates
//! (quoin-style @ main 2026-11-04, gpui-rsx-core 0.10.2).

use std::sync::Arc;

use gpui_rsx_core::{
    ClassMode, ClassOutcome, SpacingScale, Theme, ThemeBuilder, TokenValue, analyze_class,
};
use quoin_style::{Col, Decl, parse_tailwind};

/// quoin's accepted token names (`quoin-style` `token_class`, verified
/// 2026-11-04). 17 names; `destructive` aliases `Danger`, `popover` aliases
/// `Card` in quoin's token enum.
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

/// The frozen golden table: class → RGB quoin bakes into GPUI macro output
/// today (from `Token::surface_hex()`/`text_hex()`, recorded in
/// `theme_parity_baseline.txt` §1). Stored as plain RGB — quoin's values are
/// ARGB (alpha `0xFF`), hence the `>> 8` where raw values are compared.
/// 34 entries = 17 names × bg/text.
const EXPECTED: &[(&str, u32)] = &[
    ("bg-primary", 0x63_66_f1),
    ("text-primary", 0x63_66_f1),
    ("bg-secondary", 0x1e_29_3b),
    ("text-secondary", 0x94_a3_b8),
    ("bg-background", 0x0f_17_2a),
    ("text-background", 0xfa_fa_fa),
    ("bg-foreground", 0xfa_fa_fa),
    ("text-foreground", 0xfa_fa_fa),
    ("bg-muted", 0x1e_29_3b),
    ("text-muted", 0x94_a3_b8),
    ("bg-muted-foreground", 0x1e_29_3b),
    ("text-muted-foreground", 0x94_a3_b8),
    ("bg-accent", 0x63_66_f1),
    ("text-accent", 0x63_66_f1),
    ("bg-info", 0x1e_3a_5f),
    ("text-info", 0x3b_82_f6),
    ("bg-warning", 0x42_20_06),
    ("text-warning", 0xea_b3_08),
    ("bg-danger", 0x45_0a_0a),
    ("text-danger", 0xef_44_44),
    ("bg-destructive", 0x45_0a_0a),
    ("text-destructive", 0xef_44_44),
    ("bg-success", 0x14_53_2d),
    ("text-success", 0x22_c5_5e),
    ("bg-border", 0x1e_29_3b),
    ("text-border", 0xe2_e8_f0),
    ("bg-input", 0x1e_29_3b),
    ("text-input", 0x1e_29_3b),
    ("bg-ring", 0x63_66_f1),
    ("text-ring", 0x1e_29_3b),
    ("bg-card", 0x1e_29_3b),
    ("text-card", 0x1e_29_3b),
    ("bg-popover", 0x1e_29_3b),
    ("text-popover", 0x1e_29_3b),
];

/// gpui-component@0.8.1 `ThemeColor` field per quoin token name — verified
/// against gpui-rsx's generated preset table (`theme_parity_baseline.txt` §3).
/// `card` has no field; everything else resolves through the live preset.
const PRESET_FIELD: &[(&str, Option<&str>)] = &[
    ("primary", Some("primary")),
    ("secondary", Some("secondary")),
    ("background", Some("background")),
    ("foreground", Some("foreground")),
    ("muted", Some("muted")),
    ("muted-foreground", Some("muted_foreground")),
    ("accent", Some("accent")),
    ("info", Some("info")),
    ("warning", Some("warning")),
    ("danger", Some("danger")),
    ("destructive", Some("danger")),
    ("success", Some("success")),
    ("border", Some("border")),
    ("input", Some("input")),
    ("ring", Some("ring")),
    ("card", None),
    ("popover", Some("popover")),
];

/// `quoin`-side resolution of a single token class → (`RGB`, `is_text_role`).
fn quoin_baked_rgb(class: &str) -> Option<(u32, bool)> {
    let parsed = parse_tailwind(class);
    if !parsed.unknown.is_empty() {
        return None;
    }
    for decl in parsed.base.iter().chain(parsed.hover.iter()) {
        match decl {
            Decl::Background(Col::Token(t)) => return Some((t.surface_hex().value >> 8, false)),
            Decl::TextColor(Col::Token(t)) => return Some((t.text_hex().value >> 8, true)),
            _ => {}
        }
    }
    None
}

/// Text-role classes of asymmetric tokens resolve through a `<name>-fg`
/// static token (the mechanism verified in `theme_parity_baseline.txt` §2).
fn class_for_theme(class: &str) -> String {
    if let Some(rest) = class.strip_prefix("text-") {
        let surface = quoin_baked_rgb(&format!("bg-{rest}")).map_or(surface_sentinel(), |(h, _)| h);
        let text = quoin_baked_rgb(class).map_or(surface_sentinel(), |(h, _)| h);
        if text != surface {
            return format!("text-{rest}-fg");
        }
    }
    class.to_string()
}

fn surface_sentinel() -> u32 {
    u32::MAX
}

/// Byte-parity theme: one static color token per quoin name, carrying quoin's
/// frozen hex.
fn parity_theme() -> Theme {
    let mut builder = ThemeBuilder::default().scale(SpacingScale::Tailwind);
    for name in QUOIN_TOKEN_NAMES {
        let (surface, _) = quoin_baked_rgb(&format!("bg-{name}")).unwrap_or_else(|| {
            panic!("quoin no longer accepts bg-{name} — update QUOIN_TOKEN_NAMES")
        });
        builder = builder.color(
            name.to_string(),
            TokenValue::Color {
                hex: surface,
                alpha: false,
            },
        );
        let (text, is_text) = quoin_baked_rgb(&format!("text-{name}"))
            .unwrap_or_else(|| panic!("quoin no longer accepts text-{name}"));
        assert!(
            is_text,
            "text-{name} did not parse into a text-role token decl"
        );
        if text != surface {
            builder = builder.color(
                format!("{name}-fg"),
                TokenValue::Color {
                    hex: text,
                    alpha: false,
                },
            );
        }
    }
    builder.build()
}

/// Live-theming theme: the `gpui-component` preset shape.
fn preset_theme() -> Theme {
    let mut builder = ThemeBuilder::default().scale(SpacingScale::Tailwind);
    for (name, field) in PRESET_FIELD {
        if let Some(field) = field {
            builder = builder.color(
                (*name).to_string(),
                TokenValue::Expr(format!("cx.theme().colors.{field}")),
            );
        }
    }
    builder.build()
}

fn expand(class: &str, theme: &Theme) -> ClassOutcome {
    let mode = ClassMode::PERMISSIVE
        .with_scale(SpacingScale::Tailwind)
        .with_theme(Some(Arc::new(theme.clone())));
    analyze_class(class, mode).outcome
}

/// Extracts the RGB decimal from a gpui-rsx expansion like
/// `. bg (rgb (6514417u32))` (the verified `TokenStream` rendering shape).
fn rgb_of_expansion(expansion: &str) -> Option<u32> {
    let start = expansion.find("rgb")?;
    let rest = &expansion[start + 3..];
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse::<u32>().ok()
}

// ── 1. Drift guard on quoin's palette ───────────────────────────────────────
#[test]
fn quoin_baked_hexes_match_frozen_golden_table() {
    let mut failures = Vec::new();
    for (class, expected) in EXPECTED {
        match quoin_baked_rgb(class) {
            Some((hex, _)) if hex == *expected => {}
            other => failures.push(format!(
                "`{class}`: golden {expected:#08x}, quoin now resolves {other:?}"
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "quoin's baked GPUI palette drifted from the golden table:\n  {}",
        failures.join("\n  ")
    );
}

// ── 2. Machinery guard: byte-parity through gpui-rsx's Theme ────────────────
#[test]
fn gpui_rsx_theme_expands_all_34_token_classes_to_same_hex() {
    let theme = parity_theme();
    let mut failures = Vec::new();
    for (class, expected) in EXPECTED {
        let mapped = class_for_theme(class);
        match expand(&mapped, &theme) {
            ClassOutcome::Supported(ts) => {
                let rgb = rgb_of_expansion(&format!("{ts}"));
                if rgb != Some(*expected) {
                    failures.push(format!(
                        "`{class}` (as `{mapped}`): expected {expected:#08x}, expansion `{ts}`"
                    ));
                }
            }
            other => failures.push(format!(
                "`{class}` (as `{mapped}`): not Supported — {other:?}"
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "gpui-rsx theme expansion regressed (was 34/34 on 2026-11-04):\n  {}",
        failures.join("\n  ")
    );
}

// ── 3. Byte-level golden on the verified expansion shape ────────────────────
#[test]
fn representative_expansions_keep_verified_shape() {
    let theme = parity_theme();
    // Recorded verbatim from `theme_parity_baseline.txt` §2 (whitespace-
    // normalized). If any of these fail, the emitted *shape* changed —
    // downstream consumers of the token stream must be reviewed.
    let goldens: &[(&str, &str)] = &[
        ("bg-primary", ".bg(rgb(6514417u32))"),
        ("text-secondary-fg", ".text_color(rgb(9741240u32))"),
        ("bg-secondary", ".bg(rgb(1976635u32))"),
        ("bg-danger", ".bg(rgb(4524554u32))"),
        ("bg-card", ".bg(rgb(1976635u32))"),
    ];
    for (class, normalized_expected) in goldens {
        match expand(class, &theme) {
            ClassOutcome::Supported(ts) => {
                let norm: String = format!("{ts}")
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                assert_eq!(
                    &norm, normalized_expected,
                    "expansion shape drifted for `{class}`"
                );
            }
            other => panic!("`{class}`: not Supported — {other:?}"),
        }
    }
}

// ── 4. Live-preset expansion shape ──────────────────────────────────────────
#[test]
fn preset_resolves_16_of_17_names_through_live_theme() {
    let theme = preset_theme();
    for (name, field) in PRESET_FIELD {
        let class = format!("bg-{name}");
        match (*field, expand(&class, &theme)) {
            (Some(field), ClassOutcome::Supported(ts)) => {
                let norm: String = format!("{ts}")
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                assert!(
                    norm.contains(&format!("cx.theme().colors.{field}")),
                    "`{class}` no longer resolves through cx.theme().colors.{field}: `{ts}`"
                );
            }
            (Some(field), other) => {
                panic!("`{class}` lost its preset field `{field}` — {other:?}")
            }
            (None, outcome) => {
                // `card`: must NOT silently expand to a wrong color. If a
                // future gpui-rsx preset grows a `card` field, flip this arm
                // to assert the live expansion.
                assert!(
                    !matches!(outcome, ClassOutcome::Supported(_)),
                    "`bg-card` unexpectedly expanded without a preset field"
                );
            }
        }
    }
}

// ── 5. `[colors.card]` workaround behaves as documented ─────────────────────
#[test]
fn card_workaround_via_preset_expr_expands_to_popover() {
    let theme = ThemeBuilder::default()
        .scale(SpacingScale::Tailwind)
        .color(
            "card",
            TokenValue::Expr("cx.theme().colors.popover".to_string()),
        )
        .build();
    match expand("bg-card", &theme) {
        ClassOutcome::Supported(ts) => {
            let norm: String = format!("{ts}")
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            assert!(
                norm.contains("cx.theme().colors.popover"),
                "`bg-card` workaround changed semantics: `{ts}`"
            );
        }
        other => panic!("`bg-card` with a `[colors.card]` expr: not Supported — {other:?}"),
    }
}
