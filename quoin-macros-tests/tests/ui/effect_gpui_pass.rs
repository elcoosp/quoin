use quoin_macros::effect;
fn main() {
    let x = 42;
    // Legacy syntax still works
    effect! { watch: [x], || println!("{}", x) }
    // New structured syntax
    effect! { deps: [x], run: || println!("{}", x) }
    // cleanup not supported in GPUI, so the line below is removed
}
