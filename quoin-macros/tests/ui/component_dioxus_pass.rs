use quoin_macros::component;
use quoin::ReactiveContext;
use quoin_dioxus::DioxusContext;
use dioxus::prelude::*;

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
