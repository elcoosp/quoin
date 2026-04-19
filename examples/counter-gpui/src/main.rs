use gpui::*;
use gpui_platform::application;

struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e2e2e))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(format!("Count: {}", self.count))
                    .child(
                        div()
                            .id("increment-btn")
                            .px_4()
                            .py_2()
                            .bg(rgb(0x4e4e4e))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x6e6e6e)))
                            .child("Increment")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    this.count += 1;
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_| Counter { count: 0 })
        })
        .unwrap();
        cx.activate(true);
    });
}
