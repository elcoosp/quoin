use dioxus::prelude::Element; // Required for the let _ type annotation
use dioxus::prelude::*;
use quoin_macros::quoin_render; // Must be explicitly imported

fn main() {
    let _: Element = quoin_render! {
        div(class: "container") {
            "Hello Dioxus"
        }
    };
}
