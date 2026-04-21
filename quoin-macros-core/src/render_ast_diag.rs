//! Diagnostic helpers for render! argument validation.

use syn::Ident;

const KNOWN_ARGS: &[&str] = &[
    // Standard HTML-ish
    "class", "id", "on_click", "on_input", "on_change", "on_submit",
    "children", "placeholder", "value", "primary", "ghost", "destructive",
    "active", "rows", "striped", "label", "render", "key",
    "index", "href", "target", "src", "alt", "disabled", "required",
    "type", "name", "for", "title", "role", "tabindex", "autofocus",
    // Mouse events
    "on_mouse_down", "on_mouse_up", "on_mouse_enter", "on_mouse_leave",
    // virtual_list args
    "estimated_height", "items",
    // dropdown_menu args
    "trigger",
    // clipboard_button args
    "copy_text",
    // data_table column args
    "sortable", "width", "resizable", "selectable",
    // data_table args
    "on_sort", "bordered", "size",
    // Navigation
    "navigate_to",
    // Passthrough
    "cfg",
];

const KNOWN_ELEMENTS: &[&str] = &[
    "div", "h1", "h2", "h3", "p", "text", "span", "button", "input",
    "label", "img", "a", "ul", "ol", "li", "hr", "br", "textarea",
    "select", "form", "tabs", "tab", "data_table", "column",
    "virtual_list", "dropdown_menu", "rich_text", "clipboard_button",
    "item", "tab_bar",
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

/// Check if an element name looks like a typo of a known element.
/// Returns `Some(suggestion)` if a close match is found.
pub fn suggest_element(name: &str) -> Option<&'static str> {
    for known in KNOWN_ELEMENTS {
        if levenshtein(name, known) <= 2 {
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
                        .filter(|k| !matches!(**k,
                            "render" | "key" | "index" | "label"
                            | "estimated_height" | "items" | "trigger" | "copy_text"
                            | "sortable" | "width" | "resizable" | "selectable"
                            | "on_sort" | "bordered" | "size" | "on_mouse_down"
                            | "on_mouse_up" | "on_mouse_enter" | "on_mouse_leave"
                            | "navigate_to" | "cfg"
                        ))
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    warnings
}
