// quoin-macros/tests/ui/render_gpui_pass.rs
use gpui::*;
use quoin_macros::quoin_render;

fn main() {
    let _ = quoin_render! {
        div(class: "flex flex-col gap-4 p-4") {
            "Hello World"
        }
    };
}
