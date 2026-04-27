use quoin_macros::quoin_render;

fn main() {
    quoin_render! {
        foobar(class: "x") { "hello" }
    };
}
