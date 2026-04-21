use quoin::prelude::*;
use leptos::view;

component! {
    TestLeptosActions {
        state {
            count: u32 = 0,
            label: String = "hello".to_string(),
        }
        fn increment() {
            count.update(|c| *c += 1);
        }
        fn set_label(new_label: String) {
            label.set(new_label);
        }
        fn multi_signal() {
            let c = count.get();
            label.set(format!("count is {}", c));
        }
        render {
            let _ = count.get();
            view! { <div></div> }
        }
    }
}

fn main() {}
