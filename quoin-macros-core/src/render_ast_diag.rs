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

    // Specific element validation with helpful messages
    match element_name {
        "data_table" => {
            if !arg_keys.iter().any(|k| k.to_string() == "rows") {
                warnings.push("data_table requires a 'rows:' argument (e.g., rows: my_data)".to_string());
            }
        }
        "virtual_list" => {
            if !arg_keys.iter().any(|k| k.to_string() == "items") {
                warnings.push("virtual_list requires an 'items:' argument (e.g., items: events)".to_string());
            }
            if !arg_keys.iter().any(|k| k.to_string() == "estimated_height") {
                warnings.push("virtual_list requires an 'estimated_height:' argument (e.g., estimated_height: 32.0)".to_string());
            }
        }
        "column" => {
            if !arg_keys.iter().any(|k| k.to_string() == "render") {
                warnings.push("column requires a 'render:' closure (e.g., render: |row: &T| row.field.clone())".to_string());
            }
            if !arg_keys.iter().any(|k| k.to_string() == "key") {
                warnings.push("column should have a 'key:' argument for sorting (e.g., key: \"field_name\")".to_string());
            }
        }
        "clipboard_button" => {
            if !arg_keys.iter().any(|k| k.to_string() == "copy_text") {
                warnings.push("clipboard_button requires a 'copy_text:' argument (e.g., copy_text: \"hello\")".to_string());
            }
        }
        "tabs" => {
            if !arg_keys.iter().any(|k| k.to_string() == "active") {
                warnings.push("tabs requires an 'active:' argument (e.g., active: current_tab.get())".to_string());
            }
            if !arg_keys.iter().any(|k| k.to_string() == "on_click") {
                warnings.push("tabs requires an 'on_click:' callback (e.g., on_click: move |i| active.set(i))".to_string());
            }
        }
        "tab" => {
            if !arg_keys.iter().any(|k| k.to_string() == "index") {
                warnings.push("tab requires an 'index:' argument (e.g., index: 0)".to_string());
            }
            if !arg_keys.iter().any(|k| k.to_string() == "label") {
                warnings.push("tab requires a 'label:' argument (e.g., label: \"Tab Name\")".to_string());
            }
        }
        "input" | "button" | "div" => {
            // These are valid without specific required args
        }
        _ => {
            // Generic validation for standard HTML elements
            let is_standard = matches!(
                element_name,
                "h1" | "h2" | "h3" | "p" | "text" | "span"
                | "a" | "img" | "label" | "ul" | "ol" | "li" | "hr"
                | "br" | "textarea" | "select" | "form"
            );

            if !is_standard {
                return warnings;
            }

            // Elements that should not have event args
            let no_event_elements = ["img", "hr", "br"];
            let has_event = arg_keys.iter().any(|k| {
                let s = k.to_string();
                s.starts_with("on_")
            });
            if no_event_elements.contains(&element_name) && has_event {
                warnings.push(format!("<{element_name}> does not support event handlers"));
            }
        }
    }

    // Check for unknown args on all elements
    let is_special = matches!(
        element_name,
        "data_table" | "virtual_list" | "column" | "clipboard_button"
        | "dropdown_menu" | "tabs" | "tab" | "item"
    );

    if !is_special {
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
                                | "estimated_height" | "items" | "copy_text"
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
    }

    warnings
}
