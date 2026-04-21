use dioxus::prelude::*;
use quoin_macros::{component, effect, quoin_render};

fn main() {}

#[component]
fn TestEffect() -> Element {
    let mut count = use_signal(|| 0);

    // Legacy syntax
    effect! { watch: [count], || println!("changed: {}", count()) }
    // New structured syntax
    effect! { deps: [count], run: || println!("changed: {}", count()) }
    // With cleanup
    effect! { deps: [count], run: || println!("run"), cleanup: || println!("cleanup") }

    quoin_render! {
        div(class: "container") {
            "Effect Test"
        }
    }
}
