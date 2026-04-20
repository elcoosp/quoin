// quoin-macros/tests/ui/effect_dioxus_pass.rs
use quoin_macros::effect;

fn main() {
    effect! { watch: [count], || println!("changed") }
}
