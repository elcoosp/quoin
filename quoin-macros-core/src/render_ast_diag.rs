//! Diagnostic helpers for render! argument validation.

use syn::Ident;

const KNOWN_ARGS: &[&str] = &[
    "open",
    "on_open_change",
    "on_value_change",
    "on_checked_change",
    "min",
    "max", "on_submit",
    "step",
    "on_change",
    "on_complete",
    "max_length",
    "on_page_change",
    "pressed",
    "on_pressed_change",
    "options",
    "force_mount",
    "current_page",
    "total_pages",
    "show_previous_next",
    "show_first_last",
    "default_size",
    "min_size",
    "max_size",
    "direction",
    "collapsible",
    "orientation",
    "for_field",
    "message",
    "variant",
    "selected",
    "on_select",
    "mode",
    "placeholder",
    "disabled",
    "checked",
    "icon_name",
    "href",
    "target",
    "on_mouse_down",
    "on_mouse_up",
    "on_mouse_enter",
    "on_mouse_leave",
    "rows",
    "striped",
    "label",
    "render",
    "key",
    "index",
    "items",
    "estimated_height",
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
    "navigate_to",
    "cfg",
    "role",
    "tabindex",
    "autofocus",
    "loading",
    // ensure we have the base ones too
    "class",
    "id",
    "style",
    "children",
    "value",
    "on_click",
    "on_input",
    "on_change",
    "primary",
    "ghost",
    "destructive",
    "active",
    "type",
    "name",
    "for",
    "title",
    "role",
    "tabindex",
    "autofocus",
    "placeholder",
    "disabled",
    "checked",
    "required",
    "src",
    "alt",
    "href",
    "target",
    "rows",
    "striped",
    "label",
    "render",
    "key",
    "index",
    "items",
    "estimated_height",
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
    "tooltip_provider",
    "tooltip_trigger",
    "tooltip_content",
    "accordion",
    "accordion_item",
    "accordion_trigger",
    "accordion_content",
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
    "item",
    "tab_bar",
    "badge",
    "styled_text",
    "icon",
    "scroll_area",
    "separator",
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
    "accordion",
    "accordion_item",
    "accordion_trigger",
    "accordion_content",
    "alert",
    "alert_title",
    "alert_description",
    "alert_dialog",
    "alert_dialog_trigger",
    "alert_dialog_overlay",
    "alert_dialog_header",
    "alert_dialog_footer",
    "alert_dialog_title",
    "alert_dialog_description",
    "alert_dialog_content",
    "alert_dialog_action",
    "alert_dialog_cancel",
    "alert_dialog_content",
    "avatar",
    "avatar_image",
    "avatar_fallback",
    "avatar_group",
    "breadcrumb",
    "breadcrumb_list",
    "breadcrumb_item",
    "breadcrumb_link",
    "breadcrumb_page",
    "breadcrumb_separator",
    "breadcrumb_ellipsis",
    "calendar",
    "card",
    "card_header",
    "card_title",
    "card_description",
    "card_content",
    "card_footer",
    "carousel",
    "carousel_content",
    "carousel_item",
    "carousel_previous",
    "carousel_next",
    "collapsible",
    "collapsible_trigger",
    "collapsible_content",
    "combobox",
    "command",
    "command_input",
    "command_list",
    "command_empty",
    "command_group",
    "command_group_heading",
    "command_item",
    "command_shortcut",
    "command_separator",
    "context_menu",
    "context_menu_trigger",
    "context_menu_content",
    "context_menu_item",
    "context_menu_separator",
    "context_menu_label",
    "context_menu_checkbox_item",
    "context_menu_radio_group",
    "context_menu_radio_item",
    "context_menu_sub",
    "context_menu_sub_content",
    "context_menu_sub_trigger",
    "context_menu_shortcut",
    "date_picker",
    "dialog",
    "dialog_trigger",
    "dialog_content",
    "dialog_header",
    "dialog_title",
    "dialog_description",
    "dialog_footer",
    "dialog_close",
    "drawer",
    "drawer_trigger",
    "drawer_content",
    "drawer_overlay",
    "drawer_portal",
    "drawer_header",
    "drawer_footer",
    "drawer_title",
    "drawer_description",
    "drawer_close",
    "hover_card",
    "label",
    "menubar",
    "navigation_menu",
    "pagination",
    "pagination_content",
    "pagination_item",
    "pagination_link",
    "pagination_previous",
    "pagination_next",
    "pagination_ellipsis",
    "popover",
    "resizable_panel_group",
    "resizable_panel",
    "resizable_handle",
    "select",
    "select_trigger",
    "select_content",
    "select_item",
    "sheet",
    "sheet_trigger",
    "sheet_content",
    "sheet_title",
    "sheet_description",
    "table",
    "textarea",
    "toggle",
    "tooltip_provider",
    "tooltip_trigger",
    "tooltip_content",
    "error_boundary",
    "lazy_component",
    "toast_provider",
    "input_otp",
    "input_otp_separator",
    "form",
    "form_field",
    "form_item",
    "form_label",
    "form_control",
    "form_message",
    "form_description",
    "alert_dialog_content",
    "alert_dialog_header",
    "alert_dialog_footer",
    "alert_dialog_title",
    "alert_dialog_description",
    "alert_dialog_action",
    "alert_dialog_cancel",
    "dropdown_menu_trigger",
    "dropdown_menu_content",
    "dropdown_menu_item",
    "context_menu_trigger",
    "context_menu_content",
    "context_menu_item",
    "context_menu_separator",
    "command_input",
    "command_list",
    "command_empty",
    "command_group",
    "command_group_heading",
    "command_item",
    "dialog_trigger",
    "dialog_content",
    "dialog_header",
    "dialog_title",
    "dialog_description",
    "dialog_footer",
    "dialog_close",
    "sheet_trigger",
    "sheet_content",
    "sheet_title",
    "sheet_description",
    "drawer_trigger",
    "drawer_content",
    "drawer_header",
    "drawer_footer",
    "drawer_title",
    "drawer_description",
    "drawer_close",
    "carousel_content",
    "carousel_item",
    "carousel_previous",
    "carousel_next",
    "collapsible_trigger",
    "collapsible_content",
    "pagination_content",
    "pagination_item",
    "pagination_link",
    "pagination_previous",
    "pagination_next",
    "pagination_ellipsis",
    "select_trigger",
    "select_content",
    "select_item",
    "form_field",
    "form_item",
    "form_label",
    "form_control",
    "form_message",
    "form_description",
    "tooltip_provider",
    "tooltip_trigger",
    "tooltip_content",
    "menu_item",
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
        "checkbox" => {
            // checked and on_checked_change are optional
        }
        "switch" => {
            // checked and on_checked_change are optional
        }
        "radio_group" => {
            // wrapper element, no required args
        }
        "radio" => {
            // value is expected for meaningful radio groups
        }
        "slider" => {
            // value is expected, min/max/step are optional
        }
        "tooltip" => {
            // text is optional, trigger is optional
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
