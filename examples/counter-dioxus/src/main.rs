use counter_lib::use_counter;
use dioxus::prelude::*;
use quoin::prelude::*;

fn app() -> Element {
    // ✅ Store the context and counter in hooks – created only once.
    let ctx = use_hook(DioxusContext::new);
    let counter = use_hook(|| use_counter(&ctx));

    // Extract signal values outside rsx! to avoid Dioxus Readable trait conflict
    let count = counter.count.get();

    rsx! {
        div {
            "Count: {count}"
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
