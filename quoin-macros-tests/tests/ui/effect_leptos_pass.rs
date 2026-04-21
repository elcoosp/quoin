use quoin::prelude::*;
use leptos::{view, IntoView, prelude::{RwSignal, Get, ElementChild}};

#[leptos::component]
fn TestEffect() -> impl IntoView {
    let count = RwSignal::new(0);

    effect! { watch: [count], || println!("{}", count.get()) }

    view! { <div>"Effect Test"</div> }
}

fn main() {}
