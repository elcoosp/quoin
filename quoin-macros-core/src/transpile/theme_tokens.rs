//! Theme token → CSS class mapping + GPUI color resolution.
//!
//! This module serves two purposes:
//!
//! 1. **CSS class mapping** (Leptos/Dioxus): Maps theme token names to shadcn
//!    CSS classes (e.g. `"primary"` → `"bg-primary"`, `"text-primary"`).
//!    Used by emitters when a `color:` arg contains a known token string.
//!
//! 2. **GPUI color resolution**: Maps Tailwind-style theme classes to GPUI
//!    `Hsla`/`Rgba` values. Used by the Tailwind transpiler to convert classes
//!    like `text-primary` or `bg-destructive` into `.text_color(gpui::rgb(...))`
//!    calls.
//!
//! When the shadcn CSS variables are available (via shadcn-ui's base styles),
//! classes like `text-primary` resolve at runtime via `--primary`. In GPUI,
//! which has no CSS variable system, we resolve them here to hardcoded colors
//! matching the default shadcn dark theme.

/// Map a theme token name to a CSS background class.
#[must_use]
pub fn token_to_bg_class(token: &str) -> Option<&'static str> {
    match token {
        "primary" => Some("bg-primary"),
        "secondary" => Some("bg-secondary"),
        "background" => Some("bg-background"),
        "foreground" => Some("bg-foreground"),
        "muted" => Some("bg-muted"),
        "muted-foreground" => Some("bg-muted-foreground"),
        "accent" => Some("bg-accent"),
        "info" => Some("bg-info"),
        "warning" => Some("bg-warning"),
        "danger" => Some("bg-danger"),
        "destructive" => Some("bg-destructive"),
        "success" => Some("bg-green-600"),
        "border" => Some("bg-border"),
        "input" => Some("bg-input"),
        "ring" => Some("bg-ring"),
        "card" => Some("bg-card"),
        "popover" => Some("bg-popover"),
        _ => None,
    }
}

/// Map a theme token name to a CSS text color class.
#[must_use]
pub fn token_to_text_class(token: &str) -> Option<&'static str> {
    match token {
        "primary" => Some("text-primary"),
        "secondary" => Some("text-secondary"),
        "foreground" => Some("text-foreground"),
        "muted" => Some("text-muted"),
        "muted-foreground" => Some("text-muted-foreground"),
        "accent" => Some("text-accent"),
        "info" => Some("text-info"),
        "warning" => Some("text-warning"),
        "danger" => Some("text-danger"),
        "destructive" => Some("text-destructive"),
        "success" => Some("text-green-500"),
        "border" => Some("text-border"),
        _ => None,
    }
}

/// Try to resolve a color arg value to a CSS background class.
///
/// If the value is a string literal matching a known theme token (e.g. `"primary"`),
/// returns `Some("bg-primary")`. Otherwise returns `None` so the caller can
/// fall back to inline `style="background-color: …"`.
#[must_use]
pub fn try_resolve_bg_class(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        let val = s.value();
        if let Some(cls) = token_to_bg_class(&val) {
            return Some(cls.to_string());
        }
    }
    None
}

/// Try to resolve a color arg value to a CSS text color class.
#[must_use]
pub fn try_resolve_text_class(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        let val = s.value();
        if let Some(cls) = token_to_text_class(&val) {
            return Some(cls.to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// GPUI color resolution
// ---------------------------------------------------------------------------

/// Colors matching shadcn's default dark theme.
/// Used by the GPUI Tailwind transpiler to resolve theme-aware classes.
#[cfg(feature = "gpui")]
#[must_use]
pub fn try_resolve_gpui_color(class: &str) -> Option<proc_macro2::TokenStream> {
    use quote::quote;

    // shadcn-style dark palette, named once and referenced by every token.
    const INDIGO_500: &str = "0x6366f1";
    const SLATE_50: &str = "0xfafafa";
    const SLATE_200: &str = "0xe2e8f0";
    const SLATE_400: &str = "0x94a3b8";
    const SLATE_500: &str = "0x64748b";
    const SLATE_800: &str = "0x1e293b";
    const SLATE_900: &str = "0x0f172a";
    const BLUE_500: &str = "0x3b82f6";
    const BLUE_900: &str = "0x1e3a5f";
    const YELLOW_500: &str = "0xeab308";
    const YELLOW_900: &str = "0x422006";
    const RED_500: &str = "0xef4444";
    const RED_950: &str = "0x450a0a";
    const GREEN_500: &str = "0x22c55e";
    const GREEN_950: &str = "0x14532d";
    const WHITE: &str = "0xffffff";
    const BLACK: &str = "0x000000";

    /// `(class, color)` pairs; scanned linearly, order is irrelevant.
    static TOKEN_COLORS: &[(&str, &str)] = &[
        // Text colors
        ("text-primary", INDIGO_500),
        ("text-accent", INDIGO_500),
        ("text-secondary", SLATE_500),
        ("text-foreground", SLATE_50),
        ("text-muted", SLATE_400),
        ("text-muted-foreground", SLATE_400),
        ("text-info", BLUE_500),
        ("text-warning", YELLOW_500),
        ("text-danger", RED_500),
        ("text-destructive", RED_500),
        ("text-success", GREEN_500),
        ("text-border", SLATE_200),
        ("text-white", WHITE),
        ("text-black", BLACK),
        // Background colors
        ("bg-primary", INDIGO_500),
        ("bg-secondary", SLATE_800),
        ("bg-background", SLATE_900),
        ("bg-foreground", SLATE_50),
        ("bg-muted", SLATE_800),
        ("bg-muted-foreground", SLATE_800),
        ("bg-accent", SLATE_800),
        ("bg-border", SLATE_800),
        ("bg-card", SLATE_800),
        ("bg-popover", SLATE_800),
        ("bg-input", SLATE_800),
        ("bg-info", BLUE_900),
        ("bg-warning", YELLOW_900),
        ("bg-danger", RED_950),
        ("bg-destructive", RED_950),
        ("bg-success", GREEN_950),
    ];

    let color = TOKEN_COLORS
        .iter()
        .find(|(name, _)| *name == class)
        .map(|(_, color)| *color)?;

    let rgb_val: u32 = u32::from_str_radix(color.trim_start_matches("0x"), 16).unwrap_or(0);

    // Interpolate the literal directly: the old version emitted bare
    // `_r`/`_g`/`_b` identifiers into the generated code, which would not
    // compile in the consumer crate.
    if class.starts_with("text-") {
        Some(quote! { .text_color(gpui::rgb(#rgb_val)) })
    } else {
        Some(quote! { .bg(gpui::rgb(#rgb_val)) })
    }
}
