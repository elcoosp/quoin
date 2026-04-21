use dioxus::prelude::dioxus_signals;
use quoin::prelude::*;
component! {
    TestDioxus {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            let _ = "Hello Dioxus";
        }
    }
}

fn main() {}
