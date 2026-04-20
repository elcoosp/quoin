use gpui::*;
use gpui_platform::application;
use quoin::ReactiveContext;
use quoin_gpui::GpuiContext;
use quoin_macros::{component, quoin_render};

component! {
    DemoApp {
        state {
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = vec![
                Person { id: 1, name: "Alice".to_string(), age: 30 },
                Person { id: 2, name: "Bob".to_string(), age: 25 },
            ],
        }

        fn select_option(&self, option: String) {
            self.selected.set(option);
        }

        render {
            quoin_render! {
                div(class: "flex flex-col gap-4 p-4") {
                    // Dropdown example
                    dropdown(trigger: quoin_render! {
                        button(class: "px-4 py-2 bg-gray-800 text-white rounded") {
                            "Choose: "
                            {selected.get()}
                        }
                    }) {
                        menu_item(label: "Option A", on_click: action!(selected => selected.set("Option A".to_string())))
                        menu_item(label: "Option B", on_click: action!(selected => selected.set("Option B".to_string())))
                    }

                    // Data table example
                    data_table(
                        rows: rows.get(),
                        striped: true,
                        adapter: _table_adapter
                    ) {
                        column(key: "name", label: "Name", width: 150.0, sortable: true) {
                            |row: &Person| { row.name.clone() }
                        }
                        column(key: "age", label: "Age", width: 100.0) {
                            |row: &Person| { row.age.to_string() }
                        }
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

struct DemoView {
    app: DemoApp,
    _ctx: GpuiContext,
}

impl Render for DemoView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.app.render(window, cx)
    }
}

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx.open_window(WindowOptions::default(), |window, window_cx| {
            window_cx.new(|cx: &mut Context<DemoView>| {
                let ctx: GpuiContext = cx.into();
                ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                let app = DemoApp::new(cx, DemoAppProps {});
                DemoView { app, _ctx: ctx }
            })
        }).unwrap();
        app_cx.activate(true);
    });
}
