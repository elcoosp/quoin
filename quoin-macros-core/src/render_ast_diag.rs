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
    "min",
    "max",
    "step",
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
    "on_checked_change",
    // Rich-elements Phase 0 (canonical shadcn Tier-1 — see
    // `emit::shadcn_spec`): the canonical `variant:`/`size:` grammar plus
    // the avatar/card surface args. `small`/`xsmall` legalise the boolean
    // button-size sugar the GPUI emitter has always documented.
    "variant",
    "description",
    "fallback",
    "small",
    "xsmall",
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
    "checkbox",
    "switch",
    "radio_group",
    "radio",
    "slider",
    "tooltip",
    "item",
    // Rich-elements Phase 0 — keep in sync with `render_ast::KNOWN_ELEMENTS`
    // (the suggestion engine must know the Tier-1 surface too).
    "card",
    "card_header",
    "card_title",
    "card_description",
    "card_content",
    "card_footer",
    "card_action",
    "alert",
    "alert_title",
    "alert_description",
    "spinner",
    "kbd",
    "avatar",
    "avatar_fallback",
];

#[must_use]
/// Suggests the closest known argument name (edit distance ≤ 2) for a
/// typo'd argument.
pub fn suggest_arg(key: &str) -> Option<&'static str> {
    KNOWN_ARGS
        .iter()
        .find(|&known| levenshtein(key, known) <= 2)
        .map(|v| v as _)
}

#[must_use]
/// Suggests the closest known element name for a typo'd element.
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
            let cost = usize::from(ac != bc);
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[a.len()][b.len()]
}

#[must_use]
/// Validates one element's arguments and returns human-readable warnings:
/// missing required args, unsupported event handlers, and unknown args with
/// typo suggestions.
pub fn check_element_args(element_name: &str, arg_keys: &[&Ident]) -> Vec<String> {
    let mut warnings = Vec::new();
    check_required_args(element_name, arg_keys, &mut warnings);
    check_standard_element_events(element_name, arg_keys, &mut warnings);
    check_unknown_args(element_name, arg_keys, &mut warnings);
    warnings
}

/// Arguments that are only valid on the special (non-`KNOWN_ARGS`) elements.
const ELEMENT_SPECIFIC_ARGS: &[&str] = &[
    "render",
    "key",
    "index",
    "label",
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
    "on_mouse_down",
    "on_mouse_up",
    "on_mouse_enter",
    "on_mouse_leave",
    "navigate_to",
    "cfg",
    "tooltip",
    "icon_name",
];

/// Elements whose arguments are validated in [`check_required_args`].
const SPECIAL_ELEMENTS: &[&str] = &[
    "data_table",
    "virtual_list",
    "column",
    "clipboard_button",
    "dropdown_menu",
    "tabs",
    "tab",
    "item",
    "icon",
    "badge",
    "styled_text",
    "scroll_area",
    "rich_text",
];

fn check_required_args(element_name: &str, arg_keys: &[&Ident], warnings: &mut Vec<String>) {
    let missing = |name: &str| !arg_keys.iter().any(|k| *k == name);
    match element_name {
        "data_table" => {
            if missing("rows") {
                warnings.push(
                    "data_table requires a 'rows:' argument (e.g., rows: my_data)".to_string(),
                );
            }
        }
        "virtual_list" => {
            if missing("items") {
                warnings.push(
                    "virtual_list requires an 'items:' argument (e.g., items: events)".to_string(),
                );
            }
            if missing("estimated_height") {
                warnings.push("virtual_list requires an 'estimated_height:' argument (e.g., estimated_height: 32.0)".to_string());
            }
        }
        "column" => {
            if missing("render") {
                warnings.push("column requires a 'render:' closure (e.g., render: |row: &T| row.field.clone())".to_string());
            }
            if missing("key") {
                warnings.push(
                    "column should have a 'key:' argument for sorting (e.g., key: \"field_name\")"
                        .to_string(),
                );
            }
        }
        "icon" => {
            if missing("icon_name") {
                warnings.push(
                    "icon requires an 'icon_name' argument (e.g., icon_name: \"calendar\")"
                        .to_string(),
                );
            }
        }
        "clipboard_button" => {
            if missing("copy_text") {
                warnings.push("clipboard_button requires a 'copy_text:' argument (e.g., copy_text: \"hello\")".to_string());
            }
        }
        "tabs" => {
            if missing("active") {
                warnings.push(
                    "tabs requires an 'active:' argument (e.g., active: current_tab.get())"
                        .to_string(),
                );
            }
            if missing("on_click") {
                warnings.push("tabs requires an 'on_click:' callback (e.g., on_click: move |i| active.set(i))".to_string());
            }
        }
        "tab" => {
            if missing("index") {
                warnings.push("tab requires an 'index:' argument (e.g., index: 0)".to_string());
            }
            if missing("label") {
                warnings.push(
                    "tab requires a 'label:' argument (e.g., label: \"Tab Name\")".to_string(),
                );
            }
        }
        // All of these (plus the standard elements below) take only optional
        // arguments; nothing to validate.
        _ => {}
    }
}

/// Standard HTML-ish elements must not attach event handlers where none is
/// supported (`img`, `hr`, `br`).
fn check_standard_element_events(
    element_name: &str,
    arg_keys: &[&Ident],
    warnings: &mut Vec<String>,
) {
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
            | "input"
            | "button"
            | "div"
    );
    if !is_standard {
        return;
    }

    let no_event_elements = ["img", "hr", "br"];
    let has_event = arg_keys.iter().any(|k| k.to_string().starts_with("on_"));
    if no_event_elements.contains(&element_name) && has_event {
        warnings.push(format!("<{element_name}> does not support event handlers"));
    }
}

fn check_unknown_args(element_name: &str, arg_keys: &[&Ident], warnings: &mut Vec<String>) {
    if SPECIAL_ELEMENTS.contains(&element_name) {
        return;
    }
    for key_ident in arg_keys {
        let key_str = key_ident.to_string();
        if KNOWN_ARGS.contains(&key_str.as_str())
            || ELEMENT_SPECIFIC_ARGS.contains(&key_str.as_str())
        {
            continue;
        }
        if let Some(suggestion) = suggest_arg(&key_str) {
            warnings.push(format!(
                "unknown argument `{key_str}` on `<{element_name}>`. Did you mean `{suggestion}`?"
            ));
        } else {
            warnings.push(format!(
                "unknown argument `{}` on `<{element_name}>`. Known args: {}",
                key_str,
                ELEMENT_SPECIFIC_ARGS.join(", ")
            ));
        }
    }
}
