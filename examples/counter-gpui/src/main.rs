use counter_lib::use_counter;
use gpui::*;
use gpui_platform::application;
use quoin::Signal;
use quoin_gpui::GpuiContext;

struct CounterView {
    counter: counter_lib::Counter<quoin_gpui::GpuiSignal<u32>>,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.counter.count.get();

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
                    .child(format!("Count: {count}"))
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x4e4e4e))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x6e6e6e)))
                            .child("Increment")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    (this.counter.increment)();
                                    cx.notify(); // ✅ manual refresh
                                }),
                            ),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|cx| {
                let ctx = GpuiContext::new(cx);
                CounterView {
                    counter: use_counter(&ctx),
                }
            })
        })
        .unwrap();
        cx.activate(true);
    });
}
