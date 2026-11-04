//! Visual todo demo — the same `quoin_render!` source shape as the other
//! backends, rendered through the Floem backend.
//!
//! Exercises the interactive surface of the backend: a bound text input, a
//! keyed `for` list, filter buttons, dynamic footer text — and the `keys`
//! contract: **Escape clears the draft** from anywhere in the window (the
//! expansion makes the root focusable, grabs focus on launch, and dispatches
//! through a single `KeyDown` listener).
//!
//! ```bash
//! cargo run --manifest-path examples/todo-floem/Cargo.toml
//! ```

use floem::prelude::*;
use quoin::prelude::*;
use todo_core::{Filter, Todo};

component! {
    pub TodoApp {
        state {
            draft: String = String::new(),
            todos: Vec<Todo> = vec![
                Todo { id: 1, title: "Try the Floem backend".to_string(), completed: false },
                Todo { id: 2, title: "Press Escape to clear the draft".to_string(), completed: true },
            ],
            filter: Filter = Filter::All,
            next_id: u64 = 3,
        }

        keys {
            "escape" => draft.set(String::new())
        }

        render {
            quoin_render! {
                div(class: "flex flex-col gap-4 p-6 w-96") {
                    h1() { "Todos" }

                    div(class: "flex flex-row gap-2") {
                        input(value: draft, placeholder: "What needs doing?",
                              on_input: |ev: quoin::ev::Input| draft.set(ev.value.to_string()))
                        button(label: "Add",
                               on_click: |_| {
                                   let title = draft.get().trim().to_string();
                                   if !title.is_empty() {
                                       todos.update(|list| {
                                           let id = next_id.get();
                                           list.push(Todo { id, title: title.clone(), completed: false });
                                           next_id.set(id + 1);
                                       });
                                       draft.set(String::new());
                                   }
                               })
                    }

                    div(class: "flex flex-row gap-2") {
                        button(label: "all", on_click: |_| filter.set(Filter::All))
                        button(label: "active", on_click: |_| filter.set(Filter::Active))
                        button(label: "completed", on_click: |_| filter.set(Filter::Completed))
                    }

                    for[todo in visible(todos.get(), filter.get()), key: todo.id] {
                        div(class: "flex flex-row gap-2") {
                            button(label: "✓",
                                   on_click: |_| {
                                       todos.update(|list| {
                                           if let Some(item) = list.iter_mut().find(|i| i.id == todo.id) {
                                               item.completed = !item.completed;
                                           }
                                       });
                                   })
                            p() { { todo.title.clone() } }
                        }
                    }

                    p() { { format!("{} left", todos.get().iter().filter(|t| !t.completed).count()) } }
                }
            }
        }
    }
}

/// The visibility rule, shared with the headless e2e suite: the same
/// `todo-core` `Filter` drives both, so this view and the tests cannot
/// drift apart.
fn visible(todos: Vec<Todo>, filter: Filter) -> Vec<Todo> {
    match filter {
        Filter::All => todos,
        Filter::Active => todos.into_iter().filter(|t| !t.completed).collect(),
        Filter::Completed => todos.into_iter().filter(|t| t.completed).collect(),
    }
}

fn main() {
    run_app!(TodoApp);
}
