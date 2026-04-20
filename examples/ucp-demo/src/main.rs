use gpui::*;
use gpui_platform::application;
use quoin::Signal;
use quoin_gpui::GpuiContext;
use quoin_macros::{component, quoin_render};

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
            let count_text = format!("Count: {}", self.count.get());
            let selected_text = format!("Selected: {}", self.selected.get());

            let people_items = {
                let rows = self.rows.get();
                rows.iter().map(|person| {
                    let text = format!("{} ({} years old)", person.name, person.age);
                    quoin_render! {
                        div(class: "p-2 bg-gray-800 rounded-md") {
                            text
                        }
                    }
                }).collect::<Vec<_>>()
            };

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white size-full") {
                    div(class: "text-2xl font-bold") {
                        "Quoin Render Demo"
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

#[derive(Clone)]
struct Person {
    id: u32,
    name: String,
    age: u32,
}

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx
            .open_window(WindowOptions::default(), |window, window_cx| {
                window_cx.new(|cx: &mut Context<DemoApp>| {
                    // Create the reactive context from the GPUI context.
                    let ctx: GpuiContext = cx.into();

                    // Wire the view to automatically refresh when any signal changes.
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));

                    DemoApp::new(cx, ctx, DemoAppProps {})
                })
            })
            .unwrap();
        app_cx.activate(true);
    });
}
