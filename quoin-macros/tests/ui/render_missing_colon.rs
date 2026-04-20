use quoin_macros::quoin_render;

fn main() {
    quoin_render! {
        div(class "flex") { // Missing colon after class
            "Hello"
        }
    };
}
