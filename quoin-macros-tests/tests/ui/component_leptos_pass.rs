use quoin::prelude::*;
use leptos::view;

component! {
    TestLeptos {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            view! { <div></div> }
        }
    }
}

fn main() {}
