// examples/counter-xilem/src/main.rs
use std::sync::Arc;

use quoin::prelude::*;
use xilem::tokio::runtime::Runtime;
use xilem::{
    EventLoop, WidgetView, WindowOptions, Xilem,
    view::{flex_col, label, text_button},
};

#[derive(Clone)]
struct MyAppState {
    #[allow(dead_code)]
    ctx: XilemContext,
    count_signal: XilemSignal<i32>,
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let runtime = Arc::new(Runtime::new().unwrap());
    let ctx = XilemContext::new(runtime);
    let count_signal = ctx.create_signal(0);

    ctx.set_update_notifier(|| {
        println!("Signal changed – request UI rebuild here");
    });

    let app_state = MyAppState { ctx, count_signal };

    Xilem::new_simple(
        app_state,
        |state: &mut MyAppState| {
            let count = state.count_signal.get();
            let increment = {
                let signal = state.count_signal.clone();
                Arc::new(move || signal.update(|v| *v += 1)) as Arc<dyn Fn() + Send + Sync>
            };
            flex_col((
                label(format!("Count: {count}")),
                text_button("Increment", move |_| {
                    increment();
                }),
            ))
        },
        WindowOptions::new("Counter with quoin-xilem"),
    )
    .run_in(EventLoop::with_user_event())
}
