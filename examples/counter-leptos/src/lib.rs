use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    let increment = move |_| set_count.update(|c| *c += 1);
    view! {
        <div>
            <p>"Count: " {move || count.get()}</p>
            <button on:click=increment>"Increment"</button>
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
