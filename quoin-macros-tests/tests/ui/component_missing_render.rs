use quoin_macros::component;
component! {
    Test {
        state {
            count: u32 = 0,
        }
        // missing render block
    }
}
fn main() {}
