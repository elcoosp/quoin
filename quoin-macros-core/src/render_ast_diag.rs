//! Diagnostic helpers for render! argument validation.

use syn::Ident;

const KNOWN_ARGS: &[&str] = &[
    "class",
    "id",
    "on_click",
    "on_input",
    "on_change",
    "on_submit",
    "children",
    "placeholder",
    "value",
    "primary",
    "ghost",
    "destructive",
    "active",
    "rows",
    "striped",
    "label",
    "render",
    "key",
    "index",
    "href",
    "target",
    "src",
    "alt",
    "disabled",
    "required",
    "type",
    "name",
    "for",
    "title",
    "role",
    "tabindex",
    "autofocus",
    "on_mouse_down",
    "on_mouse_up",
    "on_mouse_enter",
    "on_mouse_leave",
    "checked",
    "estimated_height",
    "items",
    "copy_text",
    "sortable",
    "width",
    "resizable",
    "selectable",
    "on_sort",
    "bordered",
    "size",
    "text",
    "query",
    "color",
    "direction",
    "icon_name",
    "navigate_to",
    "cfg",
    "tooltip",
    "value",
    "max",
    "indeterminate",
];

const KNOWN_ELEMENTS: &[&str] = &[
    "div",
    "h1",
    "h2",
    "h3",
    "p",
    "text",
    "span",
    "button",
    "input",
    "label",
    "img",
    "a",
    "ul",
    "ol",
    "li",
    "hr",
    "br",
    "textarea",
    "select",
    "form",
    "tabs",
    "tab",
    "data_table",
    "column",
    "virtual_list",
    "dropdown_menu",
    "rich_text",
    "clipboard_button",
    "tab_bar",
    "badge",
    "styled_text",
    "icon",
    "scroll_area",
    "skeleton",
    "skeleton_text",
    "skeleton_avatar",
    "progress",
    "item",
];

pub fn suggest_arg(key: &str) -> Option<&'static str> {
    KNOWN_ARGS
        .iter()
        .find(|&known| levenshtein(key, known) <= 2)
        .map(|v| v as _)
}

pub fn suggest_element(name: &str) -> Option<&'static str> {
    KNOWN_ELEMENTS
        .iter()
        .find(|&known| levenshtein(name, known) <= 2)
        .map(|v| v as _)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate().take(b.len() + 1) {
        *val = j;
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

pub fn check_element_args(element_name: &str, arg_keys: &[&Ident]) -> Vec<String> {
    let mut warnings = Vec::new();

    match element_name {
        "data_table" => {
            if !arg_keys.iter().any(|k| *k == "rows") {
                warnings.push(
                    "data_table requires a 'rows:' argument (e.g., rows: my_data)".to_string(),
                );
            }
        }
        "virtual_list" => {
            if !arg_keys.iter().any(|k| *k == "items") {
                warnings.push(
                    "virtual_list requires an 'items:' argument (e.g., items: events)".to_string(),
                );
            }
            if !arg_keys.iter().any(|k| *k == "estimated_height") {
                warnings.push("virtual_list requires an 'estimated_height:' argument (e.g., estimated_height: 32.0)".to_string());
            }
        }
        "column" => {
            if !arg_keys.iter().any(|k| *k == "render") {
                warnings.push("column requires a 'render:' closure (e.g., render: |row: &T| row.field.clone())".to_string());
            }
            if !arg_keys.iter().any(|k| *k == "key") {
                warnings.push(
                    "column should have a 'key:' argument for sorting (e.g., key: \"field_name\")"
                        .to_string(),
                );
            }
        }
        "icon" => {
            if !arg_keys.iter().any(|k| *k == "icon_name") {
                warnings.push(
                    "icon requires an 'icon_name' argument (e.g., icon_name: \"calendar\")"
                        .to_string(),
                );
            }
        }
        "clipboard_button" => {
            if !arg_keys.iter().any(|k| *k == "copy_text") {
                warnings.push("clipboard_button requires a 'copy_text:' argument (e.g., copy_text: \"hello\")".to_string());
            }
        }
        "tabs" => {
            if !arg_keys.iter().any(|k| *k == "active") {
                warnings.push(
                    "tabs requires an 'active:' argument (e.g., active: current_tab.get())"
                        .to_string(),
                );
            }
            if !arg_keys.iter().any(|k| *k == "on_click") {
                warnings.push("tabs requires an 'on_click:' callback (e.g., on_click: move |i| active.set(i))".to_string());
            }
        }
        "tab" => {
            if !arg_keys.iter().any(|k| *k == "index") {
                warnings.push("tab requires an 'index:' argument (e.g., index: 0)".to_string());
            }
            if !arg_keys.iter().any(|k| *k == "label") {
                warnings.push(
                    "tab requires a 'label:' argument (e.g., label: \"Tab Name\")".to_string(),
                );
            }
        }
        "progress" => {
            // value is optional (indeterminate when missing), no validation needed
        }
        "input" | "button" | "div" => {}
        _ => {
            let is_standard = matches!(
                element_name,
                "h1" | "h2"
                    | "h3"
                    | "p"
                    | "text"
                    | "span"
                    | "a"
                    | "img"
                    | "label"
                    | "ul"
                    | "ol"
                    | "li"
                    | "hr"
                    | "br"
                    | "textarea"
                    | "select"
                    | "form"
            );

            if !is_standard {
                return warnings;
            }

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

    let is_special = matches!(
        element_name,
        "data_table"
            | "virtual_list"
            | "column"
            | "clipboard_button"
            | "dropdown_menu"
            | "tabs"
            | "tab"
            | "item"
            | "icon"
            | "badge"
            | "styled_text"
            | "scroll_area"
            | "rich_text"
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
                        KNOWN_ARGS
                            .iter()
                            .filter(|k| !matches!(
                                **k,
                                "render"
                                    | "key"
                                    | "index"
                                    | "label"
                                    | "estimated_height"
                                    | "items"
                                    | "copy_text"
                                    | "sortable"
                                    | "width"
                                    | "resizable"
                                    | "selectable"
                                    | "on_sort"
                                    | "bordered"
                                    | "size"
                                    | "on_mouse_down"
                                    | "on_mouse_up"
                                    | "on_mouse_enter"
                                    | "on_mouse_leave"
                                    | "navigate_to"
                                    | "cfg"
                                    | "tooltip"
                                    | "icon_name"
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
