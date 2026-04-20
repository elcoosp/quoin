use leptos::prelude::*;
use quoin::Signal;
use quoin_leptos::LeptosContext;
use quoin_macros::component;

#[derive(Clone)]
struct Person {
    id: u32,
    name: String,
    age: u32,
}

component! {
    DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = vec![
                Person { id: 1, name: "Alice".to_string(), age: 30 },
                Person { id: 2, name: "Bob".to_string(), age: 25 },
            ],
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
            let count_text = move || format!("Count: {}", count.get());
            let selected_text = move || format!("Selected: {}", selected.get());

            let people_items = move || {
                rows.get().iter().map(|person| {
                    let text = format!("{} ({} years old)", person.name, person.age);
                    view! {
                        <div class="p-2 bg-gray-800 rounded-md">{text}</div>
                    }
                }).collect::<Vec<_>>()
            };

            view! {
                <div class="flex flex-col gap-4 p-4 bg-gray-900 text-white min-h-screen">
                    <div class="text-2xl font-bold">"Quoin UCP Demo (Leptos)"</div>
                    <div class="flex items-center gap-2">
                        <div class="text-lg">{count_text}</div>
                        <button
                            class="px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer"
                            on:click=move |_| { increment(); }
                        >
                            "Increment"
                        </button>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="text-lg">{selected_text}</div>
                        <button
                            class="px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer"
                            on:click=move |_| { select_option_a(); }
                        >
                            "Option A"
                        </button>
                        <button
                            class="px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer"
                            on:click=move |_| { select_option_b(); }
                        >
                            "Option B"
                        </button>
                    </div>
                    <div class="text-lg font-semibold">"People:"</div>
                    <div class="flex flex-col gap-1">{people_items}</div>
                </div>
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    leptos::mount::mount_to_body(|| view! { <DemoApp /> });
}
