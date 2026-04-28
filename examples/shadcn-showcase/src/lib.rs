use leptos::prelude::*;
use quoin_macros::component;
use quoin_macros::quoin_render;
use wasm_bindgen::prelude::*;

component! {
    pub ShadcnShowcase {
        state {
            category_index: usize = 0,
        }
        render {
            let categories: Vec<&str> = vec![
                "Buttons", "Inputs", "Data", "Navigation", "Forms"
            ];
            let selected = category_index.get();

            quoin_render! {
                div(class: "flex h-screen bg-gray-900 text-white overflow-hidden") {
                    div(class: "flex flex-col bg-gray-950 border-r border-gray-800 w-64 p-4 gap-2 overflow-y-auto") {
                        div(class: "text-lg font-bold mb-4") { "ShadCN Showcase" }
                        for[cat in categories.clone()] {
                            div(
                                class: format!("px-3 py-2 rounded-md cursor-pointer text-sm {}",
                                    if cat == categories[selected] { "bg-indigo-600 text-white" } else { "text-gray-400 hover:bg-gray-800 hover:text-white" }
                                ),
                                on_click: {
                                    let cat = cat.clone();
                                    let categories = categories.clone();
                                    let signal = category_index.clone();
                                    move |_| {
                                        let idx = categories.iter().position(|c| *c == cat).unwrap_or(0);
                                        signal.set(idx);
                                    }
                                }
                            ) { { cat.to_string() } }
                        }
                    }

                    div(class: "flex-1 overflow-y-auto p-6") {

                        // ── Buttons ────────────────────────────────────────
                        if[selected == 0] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Buttons" }
                                div(class: "flex flex-wrap gap-3") {
                                    button(primary: true, on_click: |_| {}) { "Primary" }
                                    button(on_click: |_| {}) { "Outline" }
                                    button(ghost: true, on_click: |_| {}) { "Ghost" }
                                    button(destructive: true, on_click: |_| {}) { "Destructive" }
                                    button(disabled: true) { "Disabled" }
                                }
                                div(class: "mt-4") {
                                    switch(checked: true) {}
                                }
                            }
                        }

                        // ── Inputs ────────────────────────────────────────
                        else if[selected == 1] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Inputs" }
                                label() { "Text Input" }
                                input(placeholder: "Type something...")
                                label() { "Textarea" }
                                textarea(placeholder: "Multiline text...")
                                label() { "Date Picker" }
                                date_picker(placeholder: "Pick a date...")
                            }
                        }

                        // ── Data Display ──────────────────────────────────
                        else if[selected == 2] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Data Display" }
                                div(class: "flex flex-wrap gap-2") {
                                    badge(color: "primary") { "Primary" }
                                    badge(color: "success") { "Success" }
                                    badge(color: "destructive") { "Destructive" }
                                    badge() { "Default" }
                                }
                                progress(value: 65.0, class: "w-full")
                                div(class: "flex gap-2") {
                                    skeleton() {}
                                    skeleton_text() {}
                                    skeleton_avatar() {}
                                }
                                separator() {}
                                calendar() {}
                            }
                        }

                        // ── Navigation ────────────────────────────────────
                        else if[selected == 3] {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Navigation" }
                                tabs(active: 0, on_click: |_| {}) {
                                    tab(index: 0, label: "Overview")
                                    tab(index: 1, label: "Details")
                                    tab(index: 2, label: "Settings")
                                }
                                breadcrumb() {
                                    breadcrumb_list() {
                                        breadcrumb_item() { breadcrumb_link(href: "#") { "Home" } }
                                        breadcrumb_separator() {}
                                        breadcrumb_item() { breadcrumb_link(href: "#") { "Components" } }
                                        breadcrumb_separator() {}
                                        breadcrumb_item() { breadcrumb_page() { "Current" } }
                                    }
                                }
                                pagination(current_page: 1, total_pages: 5)
                            }
                        }

                        // ── Forms ─────────────────────────────────────────
                        else {
                            div(class: "flex flex-col gap-6") {
                                div(class: "text-2xl font-bold") { "Forms" }
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
