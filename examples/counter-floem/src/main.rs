//! Visual counter demo — the same `quoin_render!` source the GPUI, Leptos
//! and Dioxus demos use, rendered through the Floem backend.
//!
//! ```bash
//! cargo run --manifest-path examples/counter-floem/Cargo.toml
//! ```

use counter_lib::use_counter;
use floem::prelude::*;
use quoin::prelude::*;

fn app_view() -> impl IntoView {
    let ctx = FloemContext::new();
    let counter = use_counter(&ctx);

    let view = quoin_render! {
        div(class: "flex flex-col gap-2 p-4 w-72") {
            h1() { "Counter" }
            p() { { format!("Count: {}", counter.count.get()) } }
            button(label: "Increment", on_click: |_| counter.count.update(|c| *c += 1))
            button(label: "Reset", on_click: |_| counter.count.set(0))
        }
    };
    view
}

fn main() {
    floem::launch(app_view);
}
