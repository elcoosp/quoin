use quoin_macros::quoin_render;
use dioxus::prelude::*;

fn main() {
    let _ = quoin_render! {
        div(class: "container") {
            "Hello Dioxus"
        }
    };
}
