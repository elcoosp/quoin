// quoin-macros/tests/ui/component_dioxus_pass.rs
use dioxus::prelude::*;
use quoin::prelude::*;

component! {
    TestDioxus {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            rsx! { div {} }
        }
    }
}

fn main() {}
