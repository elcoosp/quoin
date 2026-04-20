use counter_lib::use_counter;
use dioxus::prelude::*;
use quoin::Signal;
use quoin_dioxus::DioxusContext;

fn app() -> Element {
    let ctx = DioxusContext::new();
    let counter = use_counter(&ctx);

    rsx! {
        div {
            "Count: {counter.count.get()}"
            button {
                onclick: move |_| (counter.increment)(),
                "Increment"
            }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
