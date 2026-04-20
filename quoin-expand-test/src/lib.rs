use quoin_macros::quoin_render;
use leptos::prelude::*;

pub fn test_view() -> impl IntoView {
    quoin_render! {
        div(class: "test") {
            "Hello"
        }
    }
}
