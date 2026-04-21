use quoin::prelude::*;
use leptos::{view, IntoView, prelude::{RwSignal, Get, ElementChild}};

#[leptos::component]
fn TestEffect() -> impl IntoView {
    let count = RwSignal::new(0);

    // Legacy syntax
    effect! { watch: [count], || println!("{}", count.get()) }
    // New structured syntax
    effect! { deps: [count], run: || println!("{}", count.get()) }
    // With cleanup
    effect! { deps: [count], run: || println!("run"), cleanup: || println!("cleanup") }

    view! { <div>"Effect Test"</div> }
}

fn main() {}
