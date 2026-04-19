
use counter_lib::use_counter;
use leptos::*;
use quoin_leptos::LeptosContext;

#[component]
fn App() -> impl IntoView {
    let ctx = LeptosContext::new();
    let counter = use_counter(&ctx);

    view! {
        <div>
            <p>"Count: " {move || counter.count.get()}</p>
            <button on:click=move |_| (counter.increment)()>
                "Increment"
            </button>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    mount_to_body(|| view! { <App/> });
}

