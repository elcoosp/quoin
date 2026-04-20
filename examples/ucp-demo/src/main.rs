use gpui::*;
use gpui_platform::application;
use quoin::ReactiveContext;
use quoin::Signal;
use quoin_gpui::GpuiContext;
use quoin_macros::component;

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
            div()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .child(div().child(format!("Count: {}", self.count.get())))
                .child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x4e4e4e))
                        .rounded_md()
                        .cursor_pointer()
                        .child("Increment")
                        .on_mouse_down(MouseButton::Left, {
                            let count = self.count.clone();
                            move |_ev, _window, _cx| {
                                Self::increment(count.clone());
                            }
                        })
                )
                .child(div().child(format!("Selected: {}", self.selected.get())))
                .child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x2563eb))
                        .rounded_md()
                        .cursor_pointer()
                        .child("Select Option A")
                        .on_mouse_down(MouseButton::Left, {
                            let selected = self.selected.clone();
                            move |_ev, _window, _cx| {
                                Self::select_option(selected.clone(), "Option A".to_string());
                            }
                        })
                )
                .child(div().child("People:"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .children(
                            self.rows.get().iter().map(|person| {
                                div().child(format!("{} - {}", person.name, person.age))
                            }).collect::<Vec<_>>()
                        )
                )
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
