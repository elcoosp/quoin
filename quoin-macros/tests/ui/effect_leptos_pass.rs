// quoin-macros/tests/ui/effect_leptos_pass.rs
use leptos::prelude::*;
use quoin_macros::effect;

fn main() {
    effect! { watch: [count], || println!("changed") }
}
