use counter_lib::use_counter;
use floem::prelude::*;
use quoin_floem::FloemContext;

fn app_view() -> impl IntoView {
    let ctx = FloemContext::new();
    let counter = use_counter(&ctx);

    v_stack((
        label(move || format!("Count: {}", counter.count.get())),
        h_stack((
            button("Increment").action(move || {
                (counter.increment)();
            }),
        )),
    ))
}

fn main() {
    floem::launch(app_view);
}
