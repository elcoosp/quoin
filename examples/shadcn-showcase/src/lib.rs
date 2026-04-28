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
            let categories: Vec<&str> = vec!["Buttons", "Inputs", "Data", "Navigation", "Overlays"];
            let selected = category_index.get();

            quoin_render! {
                div(class: "flex h-screen bg-gray-900 text-white overflow-hidden") {
                    div(class: "flex flex-col bg-gray-950 border-r border-gray-800 w-64 p-4 gap-2") {
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
                        if[selected == 0] {
                            div(class: "flex flex-col gap-4") {
                                div(class: "text-2xl font-bold") { "Buttons" }
                                button(primary: true, on_click: |_| {}) { "Primary" }
                                button(on_click: |_| {}) { "Outline" }
                            }
                        } else if[selected == 1] {
                            div(class: "flex flex-col gap-4") {
                                div(class: "text-2xl font-bold") { "Inputs" }
                                input(placeholder: "Text input...")
                            }
                        } else if[selected == 2] {
                            div(class: "flex flex-col gap-4") {
                                div(class: "text-2xl font-bold") { "Data Display" }
                                badge(color: "primary") { "Badge" }
                            }
                        } else if[selected == 3] {
                            div(class: "flex flex-col gap-4") {
                                div(class: "text-2xl font-bold") { "Navigation" }
                                tabs(active: 0, on_click: |_| {}) {
                                    tab(index: 0, label: "Overview")
                                    tab(index: 1, label: "Details")
                                }
                            }
                        } else {
                            div(class: "flex flex-col gap-4") {
                                div(class: "text-2xl font-bold") { "Overlays" }
                                tooltip(text: "Tooltip") { button(on_click: |_| {}) { "Hover" } }
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
