// examples/counter-xilem/src/main.rs
use quoin::reactive::ReactiveContext;
use quoin::signal::Signal;
use quoin_xilem::XilemContext;
use std::sync::Arc;
use xilem::{
    EventLoop, WidgetView, WindowOptions, Xilem,
    tokio::runtime::Runtime,
    view::{flex_col, label, text_button},
};

#[derive(Clone)]
struct MyAppState {
    ctx: XilemContext,
    count_signal: quoin_xilem::XilemSignal<i32>,
}

fn app_logic(state: &mut MyAppState) -> impl WidgetView<MyAppState> + use<> {
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
}

fn main() -> Result<(), winit::error::EventLoopError> {
    // Create a tokio runtime (Xilem already has one internally, but we need a handle)
    let runtime = Arc::new(Runtime::new().unwrap());
    let ctx = XilemContext::new(runtime);
    let count_signal = ctx.create_signal(0);

    // In a real app, you'd set the update notifier to trigger a rebuild via MessageProxy.
    // For simplicity, we just print (the signal changes will still happen, but UI won't auto-update).
    ctx.set_update_notifier(|| {
        println!("Signal changed – request UI rebuild here");
        // In practice, you would send a message to the driver to call `request_rebuild`.
    });

    let app_state = MyAppState { ctx, count_signal };

    Xilem::new_simple(
        app_state,
        app_logic,
        WindowOptions::new("Counter with quoin-xilem"),
    )
    .run_in(EventLoop::with_user_event())
}
