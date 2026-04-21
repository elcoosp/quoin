//! Diagnostic helpers for render! argument validation.

use syn::Ident;

const KNOWN_ARGS: &[&str] = &[
    "class", "id", "on_click", "on_input", "on_change", "on_submit",
    "children", "placeholder", "value", "primary", "ghost", "destructive",
    "active", "rows", "striped", "label", "render", "key",
    "index", "href", "target", "src", "alt", "disabled", "required",
    "type", "name", "for", "title", "role", "tabindex", "autofocus",
];

/// Check if an argument key looks like a typo of a known key.
/// Returns `Some(suggestion)` if a close match is found.
pub fn suggest_arg(key: &str) -> Option<&'static str> {
    for known in KNOWN_ARGS {
        if levenshtein(key, known) <= 2 {
            return Some(known);
        }
    }
    None
}

/// Simple Levenshtein distance for typo detection.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b.len() {
        matrix[0][j] = j;
    }

    for (i, ac) in a.iter().enumerate() {
        for (j, bc) in b.iter().enumerate() {
            let cost = if ac == bc { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[a.len()][b.len()]
}

/// Emit an error for unrecognized arguments on well-known elements.
pub fn check_element_args(element_name: &str, arg_keys: &[&Ident]) -> Vec<String> {
    let mut warnings = Vec::new();

    let is_standard = matches!(
        element_name,
        "div" | "h1" | "h2" | "h3" | "p" | "text" | "span" | "button"
        | "input" | "a" | "img" | "label" | "ul" | "ol" | "li" | "hr"
        | "br" | "textarea" | "select" | "form"
    );

    if !is_standard {
        return warnings;
    }

    for key_ident in arg_keys {
        let key_str = key_ident.to_string();
        if !KNOWN_ARGS.contains(&key_str.as_str()) {
            if let Some(suggestion) = suggest_arg(&key_str) {
                warnings.push(format!(
                    "unknown argument `{}` on `<{element_name}>`. Did you mean `{}`?",
                    key_str, suggestion
                ));
            } else {
                warnings.push(format!(
                    "unknown argument `{}` on `<{element_name}>`. Known args: {}",
                    key_str,
                    KNOWN_ARGS.iter()
                        .filter(|k| !matches!(**k, "render" | "key" | "index" | "label"))
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    warnings
}
