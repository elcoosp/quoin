//! Theme-parity probe: can gpui-rsx's `Theme` replace quoin's custom token system?
//!
//! Three checks:
//! 1. Every quoin token class (`bg-primary`, `text-muted`, …) parsed by quoin-style,
//!    resolved to its baked shadcn-dark hex.
//! 2. The same hexes registered in a gpui-rsx `Theme` (static, byte-parity mode):
//!    does `analyze_class` expand every quoin token class to the same hex?
//! 3. Preset emulation (`gpui-component`): the same class names resolving to
//!    `cx.theme().colors.<field>` expressions (live theming), and which quoin
//!    tokens have no preset counterpart.

use std::sync::Arc;

use gpui_rsx_core::{
    ClassMode, ClassOutcome, SpacingScale, Theme, ThemeBuilder, TokenValue, analyze_class,
};
use quoin_style::{Col, Decl, Token, parse_tailwind};

/// The token names quoin's class parser accepts (`bg-`/`text-` prefixed).
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

/// gpui-component@0.8.1 `ThemeColor` fields matching each quoin token.
/// `card` has no counterpart (verified against GPUI_COMPONENT_THEME_COLORS).
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

fn quoin_token_hex(class: &str) -> Option<(Token, u32, &'static str)> {
    let parsed = parse_tailwind(class);
    if !parsed.unknown.is_empty() {
        return None;
    }
    for decl in parsed.base.iter().chain(parsed.hover.iter()) {
        let (token, text_role) = match decl {
            Decl::Background(Col::Token(t)) => (*t, false),
            Decl::TextColor(Col::Token(t)) => (*t, true),
            _ => continue,
        };
        let hex = if text_role {
            token.text_hex()
        } else {
            token.surface_hex()
        };
        let role = if text_role { "text" } else { "surface" };
        return Some((token, hex.value, role));
    }
    None
}

/// Builds the byte-parity theme: one static color token per quoin token name,
/// carrying quoin's baked hex for that role. Asymmetric tokens (surface != text)
/// get a second `<name>-fg` token holding the text-role hex, mirroring what a
/// static rsx.toml would look like.
fn parity_theme() -> Theme {
    let mut builder = ThemeBuilder::default().scale(SpacingScale::Tailwind);
    for name in QUOIN_TOKEN_NAMES {
        let bg_class = format!("bg-{name}");
        let text_class = format!("text-{name}");
        let Some((_, surface_hex, _)) = quoin_token_hex(&bg_class) else {
            continue;
        };
        builder = builder.color(
            name.to_string(),
            TokenValue::Color {
                hex: surface_hex >> 8,
                alpha: false,
            },
        );
        if let Some((_, text_hex, _)) = quoin_token_hex(&text_class) {
            if text_hex != surface_hex {
                builder = builder.color(
                    format!("{name}-fg"),
                    TokenValue::Color {
                        hex: text_hex >> 8,
                        alpha: false,
                    },
                );
            }
        }
    }
    builder.build()
}

/// Builds the live-theming theme: every preset-mappable quoin token resolves
/// through `cx.theme().colors.<field>` (the `gpui-component` preset behavior).
fn preset_theme() -> Theme {
    let mut builder = ThemeBuilder::default().scale(SpacingScale::Tailwind);
    for (name, field) in PRESET_FIELD {
        if let Some(field) = field {
            builder = builder.color(
                name.to_string(),
                TokenValue::Expr(format!("cx.theme().colors.{field}")),
            );
        }
    }
    builder.build()
}

fn expand_with_theme(class: &str, theme: &Theme) -> String {
    let mode = ClassMode::PERMISSIVE
        .with_scale(SpacingScale::Tailwind)
        .with_theme(Some(Arc::new(theme.clone())));
    match analyze_class(class, mode).outcome {
        ClassOutcome::Supported(ts) => format!("SUPPORTED {ts}"),
        ClassOutcome::PassThrough { tokens, .. } => format!("PASSTHRU {tokens}"),
        ClassOutcome::Ignored { message } => format!("IGNORED {message}"),
        ClassOutcome::Error { message } => format!("ERROR {message}"),
        _ => "OTHER".into(),
    }
}

fn hex_of_expansion(expansion: &str) -> Option<u32> {
    // gpui-rsx emits colors as `rgb (6514417u32)` (decimal; spacing from
    // TokenStream rendering) or `0x…`.
    if let Some(start) = expansion.find("rgb") {
        let rest = &expansion[start + 3..];
        let digits: String = rest
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        return digits.parse::<u32>().ok();
    }
    let start = expansion.find("0x")?;
    let hex: String = expansion[start + 2..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();
    u32::from_str_radix(&hex, 16).ok()
}

fn main() {
    println!("=== 1. quoin token classes -> baked GPUI hex ===");
    let mut parity_table: Vec<(String, u32)> = Vec::new();
    for name in QUOIN_TOKEN_NAMES {
        for role in ["bg", "text"] {
            let class = format!("{role}-{name}");
            match quoin_token_hex(&class) {
                Some((token, hex, quoin_role)) => {
                    println!("  {class:28} -> {token:?} ({quoin_role}) {hex:#08x}");
                    parity_table.push((class, hex >> 8));
                }
                None => println!("  {class:28} -> NOT A QUOIN TOKEN CLASS"),
            }
        }
    }

    println!("\n=== 2. Static parity: same hexes in a gpui-rsx Theme ===");
    let theme = parity_theme();
    let mut mismatches = 0usize;
    for (class, quoin_hex) in &parity_table {
        // The text-role classes resolve through the `<name>-fg` token when the
        // two roles differ, so rewrite those before comparing.
        let mapped = if let Some(rest) = class.strip_prefix("text-") {
            let bg = format!("bg-{rest}");
            let (_, surface, _) = quoin_token_hex(&bg).unwrap();
            let (_, text, _) = quoin_token_hex(class).unwrap();
            if text == surface {
                class.clone()
            } else {
                format!("text-{rest}-fg")
            }
        } else {
            class.clone()
        };
        let expansion = expand_with_theme(&mapped, &theme);
        let grsx_hex = hex_of_expansion(&expansion);
        let ok = grsx_hex == Some(*quoin_hex);
        if !ok {
            mismatches += 1;
        }
        println!(
            "  {class:28} -> grsx `{mapped:26}` {} {}",
            if ok { "SAME HEX [OK]" } else { "MISMATCH [!!]" },
            expansion,
        );
    }
    println!("  mismatches: {mismatches}");

    println!("\n=== 3. Live preset: token classes -> cx.theme().colors.<field> ===");
    let preset = preset_theme();
    for (name, field) in PRESET_FIELD {
        let class = format!("bg-{name}");
        let expansion = expand_with_theme(&class, &preset);
        match field {
            Some(field) => {
                println!(
                    "  {class:28} -> {}  (field: {field})",
                    expansion.trim_start_matches("SUPPORTED ")
                );
            }
            None => {
                println!("  {class:28} -> (no preset field; declare `[colors.card]` in rsx.toml)");
            }
        }
    }

    println!("\n=== 4. Theme-driven class inventory (completion surface) ===");
    let names = theme.class_names();
    let token_class_count = parity_table.len();
    let covered = parity_table
        .iter()
        .filter(|(class, _)| {
            names.iter().any(|n| n == class) || {
                let rest = class.strip_prefix("text-").unwrap_or(class);
                names.iter().any(|n| n == &format!("text-{rest}-fg"))
            }
        })
        .count();
    println!("  quoin token classes: {token_class_count}");
    println!("  covered by parity theme: {covered}");

    println!("\n=== 5. Which tokens have asymmetric roles? ===");
    for token in Token::ALL {
        let s = token.surface_hex().value >> 8;
        let t = token.text_hex().value >> 8;
        let note = if s == t { "(same)" } else { "(DIFFER -> preset *-foreground vocabulary)" };
        println!("  {:?} surface {s:#06x} text {t:#06x} {note}", token.fn_name());
    }
}
