//! Visual todo demo — the same `quoin_render!` source shape as the other
//! backends, rendered through the Floem backend.
//!
//! Exercises the interactive surface of the M2 backend: a bound text input,
//! a keyed `for` list, checkboxes, filter tabs, and dynamic footer text.
//!
//! ```bash
//! cargo run --manifest-path examples/todo-floem/Cargo.toml
//! ```

use floem::prelude::*;
use quoin::prelude::*;
use todo_core::{Filter, TodoStore};

fn app_view() -> impl IntoView {
    let ctx = FloemContext::new();
    let store = TodoStore::new(&ctx);

    quoin_render! {
        div(class: "flex flex-col gap-4 p-6 w-96") {
            h1() { "Todos" }

            div(class: "flex flex-row gap-2") {
                input(value: store.draft, placeholder: "What needs doing?",
                      on_input: |ev: quoin::ev::Input| store.set_draft(ev.value.to_string()))
                button(label: "Add", on_click: |_| store.commit_draft())
            }

            div(class: "flex flex-row gap-2") {
                for[name in ["all", "active", "completed"], key: name] {
                    button(label: name,
                           on_click: |_| {
                               store.set_filter(match name {
                                   "active" => Filter::Active,
                                   "completed" => Filter::Completed,
                                   _ => Filter::All,
                               });
                           })
                }
            }

            p() { { format!("{}", store.active_count()) } }
        }
    }
}

fn main() {
    floem::launch(app_view);
}
