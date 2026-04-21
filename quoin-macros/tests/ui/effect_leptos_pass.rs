use leptos::prelude::*;
use quoin::Signal;
use quoin_macros::effect;

#[component]
fn TestEffect() -> impl IntoView {
    let count = RwSignal::new(0);
    effect! { watch: [count], || println!("{}", count.get()) }
    view! { <div>"Effect Test"</div> }
}

fn main() {}
