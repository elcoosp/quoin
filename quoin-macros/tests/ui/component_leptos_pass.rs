use quoin_macros::component;
use quoin::ReactiveContext;
use quoin_leptos::LeptosContext;
use leptos::prelude::*;

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
