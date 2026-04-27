use quoin_macros::component;

component! {
    TestTypo {
        rendr { // typo: should be "render"
            "hello"
        }
    }
}

fn main() {}
