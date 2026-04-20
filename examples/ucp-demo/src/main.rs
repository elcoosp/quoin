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

        fn increment(count: quoin_gpui::GpuiSignal<u32>) {
            count.update(|c| *c += 1);
        }

        fn select_option(selected: quoin_gpui::GpuiSignal<String>, option: String) {
            selected.set(option);
        }

        render {
            // Pre-render the list items so they can be collected into a Vec
            let people_rows = {
                let rows = self.rows.get();
                rows.iter().map(|person| {
                    quoin_render! {
                        div(class: "p-2 bg-gray-800 rounded") {
                            format!("{} ({} years old)", person.name, person.age)
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
                            format!("Count: {}", self.count.get())
                        }
                        button(
                            class: "px-4 py-2 bg-blue-600 text-white rounded cursor-pointer",
                            on_click: {
                                let count = self.count.clone();
                                move |_ev, _window, _cx| {
                                    Self::increment(count.clone());
                                }
                            }
                        ) {
                            "Increment"
                        }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") {
                            format!("Selected: {}", self.selected.get())
                        }
                        button(
                            class: "px-4 py-2 bg-green-600 text-white rounded cursor-pointer",
                            on_click: {
                                let selected = self.selected.clone();
                                move |_ev, _window, _cx| {
                                    Self::select_option(selected.clone(), "Option A".to_string());
                                }
                            }
                        ) {
                            "Option A"
                        }
                        button(
                            class: "px-4 py-2 bg-purple-600 text-white rounded cursor-pointer",
                            on_click: {
                                let selected = self.selected.clone();
                                move |_ev, _window, _cx| {
                                    Self::select_option(selected.clone(), "Option B".to_string());
                                }
                            }
                        ) {
                            "Option B"
                        }
                    }
                    div(class: "text-lg font-semibold") {
                        "People:"
                    }
                    div(class: "flex flex-col gap-1") {
                        people_rows
                    }
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
                    let ctx: GpuiContext = cx.into();
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                    DemoApp::new(cx, DemoAppProps {})
                })
            })
            .unwrap();
        app_cx.activate(true);
    });
}
