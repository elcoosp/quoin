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
        Person { id: 1, name: "Alice".to_string(), age: 30 },
        Person { id: 2, name: "Bob".to_string(), age: 25 },
    ]
}

#[cfg(feature = "gpui")]
component! {
    pub DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = create_initial_people(),
        }

        fn increment(&self) {
            self.count.update(|c| *c += 1);
        }

        fn select_option_a(&self) {
            self.selected.set("Option A".to_string());
        }

        fn select_option_b(&self) {
            self.selected.set("Option B".to_string());
        }

        render {
            let count_text = format!("Count: {}", self.count.get());
            let selected_text = format!("Selected: {}", self.selected.get());

            let people_items = {
                let rows = self.rows.get();
                rows.iter().map(|person| {
                    let text = format!("{} ({} years old)", person.name, person.age);
                    quoin_macros::quoin_render! {
                        div(class: "p-2 bg-gray-800 rounded-md") {
                            text
                        }
                    }
                }).collect::<Vec<_>>()
            };

            quoin_macros::quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white size-full") {
                    div(class: "text-2xl font-bold") {
                        "Quoin Render Demo (GPUI)"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") {
                            count_text
                        }
                        button(
                            class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: |this: &mut DemoApp| this.increment()
                        ) {
                            "Increment"
                        }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") {
                            selected_text
                        }
                        button(
                            class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: |this: &mut DemoApp| this.select_option_a()
                        ) {
                            "Option A"
                        }
                        button(
                            class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: |this: &mut DemoApp| this.select_option_b()
                        ) {
                            "Option B"
                        }
                    }
                    div(class: "text-lg font-semibold") {
                        "People:"
                    }
                    div(class: "flex flex-col gap-1", children: people_items)
                }
            }
        }
    }
}

#[cfg(feature = "leptos")]
component! {
    pub DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = create_initial_people(),
        }

        render {
            // Clone signals for closures
            let count_for_inc = count.clone();
            let selected_for_a = selected.clone();
            let selected_for_b = selected.clone();

            // Use quoin_render! for the main layout
            quoin_macros::quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white min-h-screen") {
                    div(class: "text-2xl font-bold") {
                        "Quoin Render Demo (Leptos)"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") {
                            format!("Count: {}", count.get())
                        }
                        button(
                            class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| count_for_inc.update(|c| *c += 1)
                        ) {
                            "Increment"
                        }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") {
                            format!("Selected: {}", selected.get())
                        }
                        button(
                            class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_for_a.set("Option A".to_string())
                        ) {
                            "Option A"
                        }
                        button(
                            class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_for_b.set("Option B".to_string())
                        ) {
                            "Option B"
                        }
                    }
                    div(class: "text-lg font-semibold") {
                        "People:"
                    }
                    // Manual rendering for the list to avoid macro edge cases
                    div(class: "flex flex-col gap-1") {
                        {
                            let rows = rows.get();
                            rows.iter().map(|person| {
                                let text = format!("{} ({} years old)", person.name, person.age);
                                view! {
                                    <div class="p-2 bg-gray-800 rounded-md">{text}</div>
                                }
                            }).collect::<Vec<_>>()
                        }
                    }
                }
            }
        }
    }
}
