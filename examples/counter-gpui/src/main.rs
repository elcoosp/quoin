
use counter_lib::use_counter;
use gpui::*;
use quoin_gpui::GpuiContext;

struct CounterView {
    counter: counter_lib::Counter<quoin_gpui::GpuiSignal<u32>>,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ctx = GpuiContext::new(cx);
        self.counter = use_counter(&ctx);

        div()
            .child(format!("Count: {}", self.counter.count.get()))
            .child(
                div()
                    .child("Increment")
                    .on_click(cx.listener(|this, _, _| {
                        (this.counter.increment)();
                    }))
            )
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| {
                let ctx = GpuiContext::new(_cx);
                CounterView {
                    counter: use_counter(&ctx),
                }
            })
        });
    });
}

