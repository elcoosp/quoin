use quoin_macros::effect;
fn main() {
    let x = 42;
    // Legacy syntax still works
    effect! { watch: [x], || println!("{}", x) }
    // New structured syntax
    effect! { deps: [x], run: || println!("{}", x) }
    // With cleanup
    let _handle = 0;
    effect! { deps: [x], run: || println!("{}", x), cleanup: || println!("cleaning up") }
}
