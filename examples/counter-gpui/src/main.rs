use counter_lib::use_counter;
use gpui::*;
use gpui_platform::application;
use quoin::Signal;
use quoin_gpui::GpuiContext;
use send_wrapper::SendWrapper;

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
                    let ctx = GpuiContext::new(cx);

                    // Weak reference to the view
                    let weak_view = cx.weak_entity();
                    // Wrap AsyncWindowContext to make it Send + Sync
                    let async_window = SendWrapper::new(window.to_async(cx));

                    ctx.set_update_notifier(move || {
                        let async_window = async_window.clone();
                        let weak_view = weak_view.clone();

                        async_window
                            .spawn(async move |cx| {
                                if let Some(view) = weak_view.upgrade() {
                                    view.update(cx, |_, cx| {
                                        cx.notify();
                                    });
                                }
                            })
                            .detach();
                    });

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
