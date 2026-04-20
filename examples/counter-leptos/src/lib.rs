use counter_lib::use_counter;
use leptos::prelude::*;
use quoin::Signal;
use quoin_leptos::LeptosContext;

#[component]
pub fn App() -> impl IntoView {
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    leptos::mount::mount_to_body(|| view! { <App/> });
}
