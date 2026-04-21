use dioxus::prelude::*;
use quoin_macros::{component, effect, quoin_render};

fn main() {}

#[component]
fn TestEffect() -> Element {
    let mut count = use_signal(|| 0);

    effect! { watch: [count], || println!("changed: {}", count()) }

    quoin_render! {
        div(class: "container") {
            "Effect Test"
        }
    }
}
