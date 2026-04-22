//! Theme token → CSS class mapping.
//!
//! Maps quoin theme token names (e.g. `"primary"`, `"destructive"`) to
//! Tailwind CSS classes (e.g. `"bg-primary"`, `"bg-destructive"`).
//!
//! This is used by emitters when a `color:` arg contains a known token
//! string literal, allowing framework-agnostic component code to reference
//! theme colors without embedding raw hex values.

/// Map a theme token name to a CSS background class.
/// Returns the class string if the token is recognized, None otherwise.
pub fn token_to_bg_class(token: &str) -> Option<&'static str> {
    match token {
        "primary"          => Some("bg-primary"),
        "secondary"        => Some("bg-secondary"),
        "background"       => Some("bg-background"),
        "foreground"       => Some("bg-foreground"),
        "muted"            => Some("bg-muted"),
        "muted-foreground" => Some("bg-muted-foreground"),
        "accent"           => Some("bg-accent"),
        "info"             => Some("bg-info"),
        "warning"          => Some("bg-warning"),
        "danger"           => Some("bg-danger"),
        "destructive"      => Some("bg-destructive"),
        "success"          => Some("bg-green-600"),
        "border"           => Some("bg-border"),
        "input"            => Some("bg-input"),
        "ring"             => Some("bg-ring"),
        "card"             => Some("bg-card"),
        "popover"          => Some("bg-popover"),
        _ => None,
    }
}

/// Map a theme token name to a CSS text color class.
/// Returns the class string if the token is recognized, None otherwise.
pub fn token_to_text_class(token: &str) -> Option<&'static str> {
    match token {
        "primary"          => Some("text-primary"),
        "secondary"        => Some("text-secondary"),
        "foreground"       => Some("text-foreground"),
        "muted"            => Some("text-muted"),
        "muted-foreground" => Some("text-muted-foreground"),
        "accent"           => Some("text-accent"),
        "info"             => Some("text-info"),
        "warning"          => Some("text-warning"),
        "danger"           => Some("text-danger"),
        "destructive"      => Some("text-destructive"),
        "success"          => Some("text-green-500"),
        "border"           => Some("text-border"),
        _ => None,
    }
}

/// Try to resolve a color arg value to a CSS background class.
///
/// If the value is a string literal matching a known theme token (e.g. `"primary"`),
/// returns `Some("bg-primary")`. Otherwise returns `None` so the caller can
/// fall back to inline `style="background-color: …"`.
pub fn try_resolve_bg_class(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = expr {
        let val = s.value();
        if let Some(cls) = token_to_bg_class(&val) {
            return Some(cls.to_string());
        }
    }
    None
}
