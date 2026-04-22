//! Framework-specific code emission for `quoin` macros.
//!
//! This module contains submodules that convert parsed ASTs into framework-native
//! Rust token streams. Each submodule is gated behind a cargo feature flag so
//! only the active framework's emitter is compiled.
//!
//! # Submodule Roles
//!
//! | Submodule              | Feature | Macro          | Description |
//! |------------------------|---------|----------------|-------------|
//! | [`gpui`]               | `gpui`  | `component!`   | Emits a GPUI struct with `Render` impl, state fields as `GpuiSignal<T>`, and a `new()` constructor that accepts `GpuiContext`. |
//! | [`leptos`]             | `leptos`| `component!`   | Emits a `#[component]` function with `LeptosContext`-based signal initialization. |
//! | [`dioxus`]             | `dioxus`| `component!`   | Emits a `#[component]` function with `DioxusContext`-based signal initialization via `use_hook`. |
//! | [`render_gpui`]        | `gpui`  | `quoin_render!`| Converts `RenderNode` AST into GPUI builder-pattern calls (`.child()`, `.on_mouse_down()`, etc.) with Tailwind transpilation. |
//! | [`render_leptos`]      | `leptos`| `quoin_render!`| Converts `RenderNode` AST into Leptos `view!` macro calls with proper attribute mapping (`on:click`, `class=`, etc.). |
//! | [`render_dioxus`]      | `dioxus`| `quoin_render!`| Converts `RenderNode` AST into Dioxus `rsx!` macro calls with native `if`/`for` syntax support. |
//! | [`run_app_gpui`]       | `gpui`  | `run_app!`     | Emits a `main()` that calls `quoin::launch()` with `open_window` and `set_view_update_notifier`. |
//! | [`run_app_leptos`]     | `leptos`| `run_app!`     | Emits a `main()` that calls `leptos::mount::mount_to_body`. |
//! | [`run_app_dioxus`]     | `dioxus`| `run_app!`     | Emits a `main()` that calls `dioxus::launch`. |

#[cfg(feature = "dioxus")]
pub mod dioxus;
#[cfg(feature = "gpui")]
pub mod gpui;
#[cfg(feature = "leptos")]
pub mod leptos;

#[cfg(feature = "dioxus")]
pub mod render_dioxus;
#[cfg(feature = "gpui")]
pub mod render_gpui;
#[cfg(feature = "leptos")]
pub mod render_leptos;

#[cfg(feature = "dioxus")]
pub mod run_app_dioxus;
#[cfg(feature = "gpui")]
pub mod run_app_gpui;
#[cfg(feature = "leptos")]
pub mod run_app_leptos;
