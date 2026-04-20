// quoin-macros/tests/ui/component_dioxus_pass.rs
use quoin_macros::component;

component! {
    TestDioxus {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            dioxus::prelude::rsx! { div {} }
        }
    }
}

fn main() {}
