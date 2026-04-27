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
pub fn try_resolve_gpui_color(class: &str) -> Option<proc_macro2::TokenStream> {
    use quote::quote;

    let color = match class {
        // Text colors
        "text-primary" => "0x6366f1",          // indigo-500
        "text-secondary" => "0x64748b",        // slate-500
        "text-foreground" => "0xfafafa",       // slate-50
        "text-muted" => "0x94a3b8",            // slate-400
        "text-muted-foreground" => "0x94a3b8", // slate-400
        "text-accent" => "0x6366f1",           // indigo-500
        "text-info" => "0x3b82f6",             // blue-500
        "text-warning" => "0xeab308",          // yellow-500
        "text-danger" => "0xef4444",           // red-500
        "text-destructive" => "0xef4444",      // red-500
        "text-success" => "0x22c55e",          // green-500
        "text-border" => "0xe2e8f0",           // slate-200
        "text-white" => "0xffffff",
        "text-black" => "0x000000",

        // Background colors
        "bg-primary" => "0x6366f1",          // indigo-500
        "bg-secondary" => "0x1e293b",        // slate-800
        "bg-background" => "0x0f172a",       // slate-900 (dark theme)
        "bg-foreground" => "0xfafafa",       // slate-50
        "bg-muted" => "0x1e293b",            // slate-800
        "bg-muted-foreground" => "0x1e293b", // slate-800
        "bg-accent" => "0x1e293b",           // slate-800
        "bg-info" => "0x1e3a5f",             // blue-900
        "bg-warning" => "0x422006",          // yellow-900
        "bg-danger" => "0x450a0a",           // red-950
        "bg-destructive" => "0x450a0a",      // red-950
        "bg-success" => "0x14532d",          // green-950
        "bg-border" => "0x1e293b",           // slate-800
        "bg-card" => "0x1e293b",             // slate-800
        "bg-popover" => "0x1e293b",          // slate-800
        "bg-input" => "0x1e293b",            // slate-800

        _ => return None,
    };

    let rgb_val: u32 = u32::from_str_radix(color, 16).unwrap_or(0);
    let _r = ((rgb_val >> 16) & 0xFF) as f32 / 255.0;
    let _g = ((rgb_val >> 8) & 0xFF) as f32 / 255.0;
    let _b = (rgb_val & 0xFF) as f32 / 255.0;

    if class.starts_with("text-") {
        Some(quote! { .text_color(gpui::rgba(_r, _g, _b, 1.0)) })
    } else {
        Some(quote! { .bg(gpui::rgba(_r, _g, _b, 1.0)) })
    }
}
