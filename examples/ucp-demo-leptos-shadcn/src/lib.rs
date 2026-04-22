//! Quoin UCP Demo — Leptos with shadcn component backing.
//!
//! This example uses the same `ucp-lib::MiniDevtools` component as the
//! plain-Leptos demo, but with the `leptos-shadcn` feature enabled on `quoin`.
//! When active, the macro emitter targets `leptos-shadcn-*` components
//! (Button, Input, Tabs, Badge, Table, Tooltip) instead of plain HTML.
//!
//! # Running
//!
//! ```sh
//! trunk serve
//! ```
//!
//! # Requirements
//!
//! The `leptos-shadcn-*` component crates must be available. See
//! `Cargo.toml` for the dependency list — uncomment them once resolved.

use leptos::prelude::*;
use ucp_lib::MiniDevtools;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    leptos::mount::mount_to_body(|| view! { <MiniDevtools /> });
}
