use quoin_macros::component;

component! {
    TestNoDefault {
        state {
            count: u32, // missing default
        }
        render {
            "hello"
        }
    }
}

fn main() {}
