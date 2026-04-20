// quoin-macros/tests/ui/effect_gpui_pass.rs
use quoin_macros::effect;

fn main() {
    let x = 42;
    effect! { watch: [x], || println!("{}", x) }
}
