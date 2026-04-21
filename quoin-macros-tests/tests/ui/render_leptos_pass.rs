use quoin_macros::quoin_render;
use leptos::prelude::*;

fn main() {
    let _ = quoin_render! {
        div(class: "container") {
            "Hello Leptos"
        }
    };
}
