use counter_lib::use_counter;
use gpui::*;
use gpui_platform::application;
use quoin::Signal;
use quoin_gpui::GpuiContext;

struct CounterView {
    counter: counter_lib::Counter<quoin_gpui::GpuiSignal<u32>>,
    _ctx: GpuiContext,
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
                                cx.listener(|this, _event, _window, _cx| {
                                    (this.counter.increment)();
                                }),
                            ),
                    ),
            )
    }
}

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx
            .open_window(WindowOptions::default(), |window, window_cx| {
                window_cx.new(|cx: &mut Context<CounterView>| {
                    // Create the reactive context from the GPUI context.
                    let ctx: GpuiContext = cx.into();

                    // Wire the view to automatically refresh when any signal changes.
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));

                    CounterView {
                        counter: use_counter(&ctx),
                        _ctx: ctx,
                    }
                })
            })
            .unwrap();

        app_cx.activate(true);
    });
}
