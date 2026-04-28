use leptos::prelude::*;
use quoin_macros::component;
use quoin_macros::quoin_render;
use wasm_bindgen::prelude::*;

component! {
    pub ShadcnShowcase {
        state {
            category_index: usize = 0,
            // Input states
            text_value: String = String::new(),
            checked: bool = false,
            switch_on: bool = true,
            slider_val: f64 = 50.0,
            selected_radio: String = "option_a".to_string(),
            otp_code: String = String::new(),
            select_val: String = "a".to_string(),
            combobox_val: String = String::new(),
            // Data states
            accordion_val: Vec<String> = vec![],
            progress_val: f64 = 65.0,
            current_page: usize = 1,
            // Overlay states
            dialog_open: bool = false,
            sheet_open: bool = false,
            drawer_open: bool = false,
            alert_dialog_open: bool = false,
            // Other states
            command_query: String = String::new(),
            calendar_date: Option<CalendarDate> = None,
        }
        render {
            let categories: Vec<&str> = vec![
                "Buttons", "Inputs", "Data Display", "Navigation",
                "Feedback", "Overlays", "Layout", "Other"
            ];
            let selected = category_index.get();

            // Pre-compute sidebar click handlers
            let sidebar_handlers: Vec<_> = categories.iter().enumerate().map(|(i, _)| {
                let signal = category_index.clone();
                move |_| signal.set(i)
            }).collect();

            let sidebar_items: Vec<_> = categories.iter().enumerate().map(|(i, &cat)| {
                (cat.to_string(), sidebar_handlers[i].clone())
            }).collect();
            let dd_trigger = quoin_render! {
                button() { "Dropdown" }
            };
            let dd_trigger = quoin_render! {
                button() { "Dropdown" }
            };
            let dd_trigger = quoin_render! {
                button() { "Dropdown" }
            };




            quoin_render! {
                div(class: "flex h-screen bg-gray-900 text-white overflow-hidden") {
                    // Sidebar
                    div(class: "flex flex-col bg-gray-950 border-r border-gray-800 w-64 p-4 gap-2 overflow-y-auto") {
                        div(class: "text-lg font-bold mb-4") { "ShadCN Showcase" }
                        for[pair in sidebar_items] {
                            div(
                                class: format!("px-3 py-2 rounded-md cursor-pointer text-sm {}",
                                    if pair.0 == categories[selected] { "bg-indigo-600 text-white" } else { "text-gray-400 hover:bg-gray-800 hover:text-white" }
                                ),
                                on_click: pair.1.clone()
                            ) { { pair.0.clone() } }
                        }
                    }
                    // Content area
                    div(class: "flex-1 overflow-y-auto p-6") {
                        // ── Buttons ────────────────────────────────────────
                        if[selected == 0] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Buttons" }
                                div(class: "flex flex-wrap gap-4") {
                                    button(primary: true, on_click: |_| {}) { "Primary" }
                                    button(on_click: |_| {}) { "Outline" }
                                    button(ghost: true, on_click: |_| {}) { "Ghost" }
                                    button(destructive: true, on_click: |_| {}) { "Destructive" }
                                    button(disabled: true) { "Disabled" }
                                }
                                div(class: "text-lg mt-4") { "Toggle" }
                                toggle(pressed: switch_on, on_change: |v| switch_on.set(v)) { "Toggle" }
                            }
                        }
                        // ── Inputs ────────────────────────────────────────
                        else if[selected == 1] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Inputs" }
                                label() { "Text Input" }
                                input(placeholder: "Type something...", value: text_value, class: "w-full max-w-md")
                                label() { "Textarea" }
                                textarea(placeholder: "Multiline text...", class: "w-full max-w-md h-24")
                                div(class: "flex items-center gap-4") {
                                    checkbox(checked: checked, on_checked_change: |v| checked.set(v))
                                    label() { format!("Checkbox: {}", if checked.get() { "on" } else { "off" }) }
                                }
                                div(class: "flex items-center gap-4") {
                                    switch(checked: switch_on, on_checked_change: |v| switch_on.set(v))
                                    label() { format!("Switch: {}", if switch_on.get() { "on" } else { "off" }) }
                                }
                                label() { "Slider" }
                                slider(value: slider_val, min: 0.0, max: 100.0, on_change: |v: String| slider_val.set(v.parse().unwrap_or(0.0)))
                                div(class: "text-sm text-gray-400") { format!("Value: {:.0}", slider_val.get()) }
                                label() { "Radio Group" }
                                radio_group(value: selected_radio, on_value_change: |v| selected_radio.set(v)) {
                                    radio(value: "option_a") { "Option A" }
                                    radio(value: "option_b") { "Option B" }
                                    radio(value: "option_c") { "Option C" }
                                }
                                label() { "OTP Input" }
                                input_otp(max_length: 4, on_complete: |code| otp_code.set(code))
                                div(class: "text-sm text-gray-400") { format!("OTP: {}", otp_code.get()) }
                                label() { "Select" }
                                select(value: select_val) {
                                    select_trigger() { "Choose..." }
                                    select_content() {
                                        select_item(value: "a") { "Item A" }
                                        select_item(value: "b") { "Item B" }
                                        select_item(value: "c") { "Item C" }
                                    }
                                }
                                label() { "Combobox" }
                                combobox(
                                    placeholder: "Search...",
                                    options: vec![
                                        ComboboxOption::new("opt1".to_string(), "Option 1"),
                                        ComboboxOption::new("opt2".to_string(), "Option 2"),
                                        ComboboxOption::new("opt3".to_string(), "Option 3"),
                                    ]
                                )
                            }
                        }
                        // ── Data Display ──────────────────────────────────
                        else if[selected == 2] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Data Display" }
                                div(class: "flex gap-2") {
                                    badge(color: "primary") { "Primary" }
                                    badge(color: "secondary") { "Secondary" }
                                    badge(color: "destructive") { "Destructive" }
                                    badge(color: "success") { "Success" }
                                    badge(color: "warning") { "Warning" }
                                    badge() { "Default" }
                                }
                                label() { "Accordion" }
                                accordion(value: accordion_val) {
                                    accordion_item(value: "item-1".to_string()) {
                                        accordion_trigger() { "Section One" }
                                        accordion_content() { div() { "Content for section one." } }
                                    }
                                    accordion_item(value: "item-2".to_string()) {
                                        accordion_trigger() { "Section Two" }
                                        accordion_content() { div() { "Content for section two." } }
                                    }
                                }
                                label() { "Progress" }
                                progress(value: progress_val, class: "w-full")
                                div(class: "text-sm text-gray-400") { format!("{:.0}%", progress_val.get()) }
                                label() { "Pagination" }
                                pagination(current_page: current_page, total_pages: 5, on_page_change: |p| current_page.set(p))
                                label() { "Skeleton" }
                                skeleton() {}
                                skeleton_text() {}
                                div(class: "flex gap-2 items-center") {
                                    skeleton_avatar() {}
                                    skeleton() {}
                                }
                                label() { "Avatar Group" }
                                avatar_group() {
                                    avatar() { avatar_fallback() { "A" } }
                                    avatar() { avatar_fallback() { "B" } }
                                    avatar() { avatar_fallback() { "C" } }
                                }
                                separator() {}
                            }
                        }
                        // ── Navigation ────────────────────────────────────
                        else if[selected == 3] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Navigation" }
                                breadcrumb() {
                                    breadcrumb_list() {
                                        breadcrumb_item() { breadcrumb_link(href: "#") { "Home" } }
                                        breadcrumb_separator() {}
                                        breadcrumb_item() { breadcrumb_link(href: "#") { "Components" } }
                                        breadcrumb_separator() {}
                                        breadcrumb_item() { breadcrumb_page() { "Current" } }
                                    }
                                }
                                pagination(current_page: 1, total_pages: 10)
                                tabs(active: 0, on_click: |_| {}) {
                                    tab(index: 0, label: "Overview")
                                    tab(index: 1, label: "Details")
                                    tab(index: 2, label: "Settings")
                                }
                            }
                        }
                        // ── Feedback ──────────────────────────────────────
                        else if[selected == 4] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Feedback" }
                                alert(variant: "default") {
                                    alert_title() { "Information" }
                                    alert_description() { "This is a default alert." }
                                }
                                alert(variant: "destructive") {
                                    alert_title() { "Error" }
                                    alert_description() { "Something went wrong!" }
                                }
                                alert(variant: "success") {
                                    alert_title() { "Success" }
                                    alert_description() { "Operation completed." }
                                }
                                alert(variant: "warning") {
                                    alert_title() { "Warning" }
                                    alert_description() { "Please check your input." }
                                }
                                button(destructive: true, on_click: |_| alert_dialog_open.set(true)) { "Show Alert Dialog" }
                                if[alert_dialog_open.get()] {
                                    alert_dialog(open: alert_dialog_open) {
                                        alert_dialog_overlay() {}
                                        alert_dialog_content() {
                                            alert_dialog_header() {
                                                alert_dialog_title() { "Are you sure?" }
                                                alert_dialog_description() { "This action cannot be undone." }
                                            }
                                            alert_dialog_footer() {
                                                alert_dialog_cancel() { "Cancel" }
                                                alert_dialog_action() { "Continue" }
                                            }
                                        }
                                    }
                                }
                                toast_provider() {}
                                tooltip(text: "This is a tooltip") {
                                    button(on_click: |_| {}) { "Hover me" }
                                }
                            }
                        }
                        // ── Overlays ──────────────────────────────────────
                        else if[selected == 5] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Overlays" }
                                button(on_click: |_| dialog_open.set(true)) { "Open Dialog" }
                                if[dialog_open.get()] {
                                    dialog(open: dialog_open) {
                                        dialog_content() {
                                            dialog_header() {
                                                dialog_title() { "Dialog Title" }
                                                dialog_description() { "Dialog description." }
                                            }
                                            dialog_footer() {
                                                dialog_close() { "Close" }
                                            }
                                        }
                                    }
                                }
                                button(on_click: |_| sheet_open.set(true)) { "Open Sheet" }
                                if[sheet_open.get()] {
                                    sheet(open: sheet_open) {
                                        sheet_content() {
                                            sheet_title() { "Sheet Title" }
                                            sheet_description() { "Sheet content." }
                                        }
                                    }
                                }
                                button(on_click: |_| drawer_open.set(true)) { "Open Drawer" }
                                if[drawer_open.get()] {
                                    drawer(open: drawer_open) {
                                        drawer_content() {
                                            drawer_header() {
                                                drawer_title() { "Drawer Title" }
                                                drawer_description() { "Drawer content." }
                                            }
                                            drawer_footer() {
                                                drawer_close() { "Close" }
                                            }
                                        }
                                    }
                                }
                                dropdown_menu(trigger: dd_trigger) {
                                    item(label: "Profile", on_click: |_| {})
                                    item(label: "Settings", on_click: |_| {})
                                    item(label: "Logout", on_click: |_| {})
                                }
                                context_menu() {
                                    context_menu_trigger() {
                                        div(class: "p-4 bg-gray-800 rounded-md cursor-pointer") { "Right-click me" }
                                    }
                                    context_menu_content() {
                                        context_menu_item() { "Copy" }
                                        context_menu_item() { "Paste" }
                                        context_menu_separator() {}
                                        context_menu_item() { "Delete" }
                                    }
                                }
                                popover() { div() { "Popover content" } }
                                hover_card() { div() { "Hover card content" } }
                            }
                        }
                        // ── Layout ────────────────────────────────────────
                        else if[selected == 6] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Layout" }
                                card(class: "w-full max-w-md") {
                                    card_header() {
                                        card_title() { "Card Title" }
                                        card_description() { "This is a card description." }
                                    }
                                    card_content() { "Card content goes here." }
                                    card_footer() { "Card footer" }
                                }
                                div(class: "flex gap-4") {
                                    card(variant: "success", class: "flex-1") {
                                        card_header() { card_title() { "Success" } }
                                        card_content() { "Success card" }
                                    }
                                    card(variant: "destructive", class: "flex-1") {
                                        card_header() { card_title() { "Destructive" } }
                                        card_content() { "Destructive card" }
                                    }
                                    card(variant: "warning", class: "flex-1") {
                                        card_header() { card_title() { "Warning" } }
                                        card_content() { "Warning card" }
                                    }
                                }
                                collapsible(open: false) {
                                    collapsible_trigger() {
                                        div(class: "cursor-pointer p-2 bg-gray-800 rounded-md") { "Click to expand" }
                                    }
                                    collapsible_content() { div() { "Collapsible content." } }
                                }
                                carousel() {
                                    carousel_content() {
                                        carousel_item() { div(class: "p-4 bg-gray-800 rounded-md") { "Slide 1" } }
                                        carousel_item() { div(class: "p-4 bg-gray-800 rounded-md") { "Slide 2" } }
                                        carousel_item() { div(class: "p-4 bg-gray-800 rounded-md") { "Slide 3" } }
                                    }
                                    carousel_previous() {}
                                    carousel_next() {}
                                }
                                resizable_panel_group(direction: "horizontal", class: "w-full h-48 border border-gray-700 rounded-md") {
                                    resizable_panel(default_size: 30.0, min_size: 20.0) { div(class: "p-2") { "Left" } }
                                    resizable_handle() {}
                                    resizable_panel(default_size: 70.0) { div(class: "p-2") { "Right" } }
                                }
                                scroll_area(class: "h-32 border border-gray-700 rounded-md p-2") {
                                    div() { "Line 1" } div() { "Line 2" } div() { "Line 3" }
                                    div() { "Line 4" } div() { "Line 5" }
                                }
                            }
                        }
                        // ── Other ─────────────────────────────────────────
                        else {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Other" }
                                calendar() {}
                                date_picker(placeholder: "Select date...")
                                command(value: command_query, on_value_change: |v| command_query.set(v)) {
                                    command_input(placeholder: "Type a command or search...")
                                    command_list() {
                                        command_empty() { "No results found." }
                                        command_group() {
                                            command_group_heading() { "Suggestions" }
                                            command_item(value: "calendar") { icon(icon_name: "calendar") { "Calendar" } }
                                            command_item(value: "search") { icon(icon_name: "search") { "Search" } }
                                            command_item(value: "settings") { icon(icon_name: "settings") { "Settings" } }
                                        }
                                    }
                                }
                                form(on_submit: |_| {}) {
                                    form_field(name: "username") {
                                        form_label(for_field: "username") { "Username" }
                                        form_control() { input(placeholder: "Enter username") }
                                        form_message(message: "Required field") {}
                                    }
                                    form_field(name: "email") {
                                        form_label(for_field: "email") { "Email" }
                                        form_control() { input(placeholder: "Enter email") }
                                        form_description() { "We'll never share your email." }
                                    }
                                }
                                error_boundary() {
                                    div(class: "p-4 bg-red-900/20 border border-red-800 rounded-md") { "Error boundary content" }
                                }
                                lazy_component(name: "SomeLazyComponent")
                            }
                        }
                    }
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <ShadcnShowcase /> });
}
