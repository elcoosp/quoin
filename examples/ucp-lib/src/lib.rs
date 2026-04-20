// examples/ucp-lib/src/lib.rs
use quoin::Signal;
use quoin_macros::component;

#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u32,
}

pub fn create_initial_people() -> Vec<Person> {
    vec![
        Person {
            id: 1,
            name: "Alice".to_string(),
            age: 30,
        },
        Person {
            id: 2,
            name: "Bob".to_string(),
            age: 25,
        },
    ]
}

// One block to rule them all. Signal updates trigger UI refreshes natively.
#[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus"))]
component! {
    pub DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = create_initial_people(),
        }

        render {
            let count_text = format!("Count: {}", count.get());
            let selected_text = format!("Selected: {}", selected.get());

            let people_items = {
                let rows = rows.get();
                rows.iter().map(|person| {
                    let text = format!("{} ({} years old)", person.name, person.age);
                    quoin_macros::quoin_render! {
                        div(class: "p-2 bg-gray-800 rounded-md") { text }
                    }
                }).collect::<Vec<_>>()
            };

            let count_inc = count.clone();
            let selected_a = selected.clone();
            let selected_b = selected.clone();

            quoin_macros::quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { count_text }
                        button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| count_inc.update(|c| *c += 1)) { "Increment" }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { selected_text }
                        button(class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_a.set("Option A".to_string())) { "Option A" }
                        button(class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_b.set("Option B".to_string())) { "Option B" }
                    }
                    div(class: "text-lg font-semibold") { "People:" }
                    div(class: "flex flex-col gap-1", children: people_items)
                }
            }
        }
    }
}
