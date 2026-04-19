use counter_lib::use_counter;
use quoin_xilem::XilemContext;
use winit::error::EventLoopError;
use xilem::view::{button, flex, label, Axis};
use xilem::{EventLoop, WidgetView, Xilem};

#[derive(Default)]
struct AppState {
    ctx: XilemContext,
}

fn app_logic(state: &mut AppState) -> impl WidgetView + use<> {
    let counter = use_counter(&state.ctx);

    flex(
        Axis::Vertical,
        (
            label(format!("Count: {}", counter.count.get())),
            button("Increment", move |_| {
                (counter.increment)();
            }),
        ),
    )
}

fn main() -> Result<(), EventLoopError> {
    let app = Xilem::new_simple(AppState::default(), app_logic, "Counter".into());
    app.run_in(EventLoop::with_user_event())
}
