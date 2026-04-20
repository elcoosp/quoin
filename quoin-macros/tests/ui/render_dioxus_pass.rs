// quoin-macros/tests/ui/render_dioxus_pass.rs
use quoin_macros::quoin_render;

fn main() {
    let _ = quoin_render! {
        div(class: "container") {
            "Hello Dioxus"
        }
    };
}
