use quoin::component;
use quoin::quoin_render;

component! {
    pub DropdownTest {
        state {
            filter: String = "all".to_string(),
        }
        render {
            let filter = filter.clone();
            quoin_render! {
                div(class: "flex flex-col") {
                    dropdown_menu(trigger: { quoin_render! { div(class: "p-2 bg-gray-700 rounded cursor-pointer") { "Filter ▾" } } }) {
                        item(label: "All", on_click: move |_| filter.clone().set("all".to_string()))
                        item(label: "Nav", on_click: move |_| filter.clone().set("nav".to_string()))
                    }
                }
            }
        }
    }
}

fn main() {}
