use quoin_macros::effect;
fn main() {
    let x = 42;
    effect! { watch: [x], || println!("{}", x) }
}
