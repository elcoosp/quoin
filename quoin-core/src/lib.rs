//! # quoin – One Reactive Core, Many Frameworks
//!
//! **quoin** provides foundational traits for building framework‑agnostic
//! reactive libraries in Rust. It abstracts over common reactive primitives—
//! signals, async executors, and cancellation—so you can write business logic
//! once and run it with GPUI, Dioxus, Leptos, or any other supported UI
//! framework.
//!
//! ## Quick Start
//!
//! Add `quoin` to your `Cargo.toml` and enable **exactly one** adapter feature:
//!
//! ```toml
//! [dependencies]
//! quoin = { version = "0.1", features = ["gpui"] }   # or "dioxus", "leptos", …
//! ```
//!
//! Then implement a framework‑agnostic hook using the [`ReactiveContext`] trait:
//!
//! ```rust,ignore
//! use quoin::{ReactiveContext, Signal};
//!
//! pub fn use_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
//!     cx.create_signal(0u32)
//! }
//! ```
//!
//! The adapter crates (`quoin-gpui`, `quoin-dioxus`, …) provide concrete
//! implementations of [`ReactiveContext`] for their respective frameworks.
//!
//! ## Feature Flags
//!
//! This crate uses feature flags to select the target UI framework. **Only one
//! framework feature may be enabled at a time.** The available features are:
//!
//! - `gpui`   – Enables compatibility with the GPUI framework.
//! - `dioxus` – Enables compatibility with Dioxus.
//! - `leptos` – Enables compatibility with Leptos.
//! - `xilem`  – Enables compatibility with Xilem.
//! - `floem`  – Enables compatibility with Floem.
//!
//! The feature flags are additive only to adapter crates; they do not change
//! the API of `quoin` itself.
//!
//! ## Core Abstractions
//!
//! - **[`ReactiveContext`]** – The entry point for creating signals and spawning
//!   asynchronous work. Each framework adapter provides its own implementation.
//! - **[`Signal`]** – A readable reactive value. Signals are `Clone` and can be
//!   passed into closures and stored across threads.
//! - **[`Executor`]** – A framework’s async task spawner. Used to run futures in
//!   the background without blocking the UI.
//! - **[`CancellationToken`]** – A cooperative cancellation mechanism for
//!   long‑running async tasks.
//!
//! ## Example: Framework‑Agnostic Async Hook
//!
//! ```rust,ignore
//! use quoin::{CancellationToken, Executor, ReactiveContext};
//!
//! pub fn use_fetch<C, F>(cx: &C, url: &str) -> C::Signal<Option<String>>
//! where
//!     C: ReactiveContext,
//!     C::Signal<Option<String>>: 'static,
//! {
//!     let signal = cx.create_signal(None);
//!     let executor = cx.executor();
//!     let token = CancellationToken::new();
//!
//!     executor.spawn({
//!         let signal = signal.clone();
//!         let url = url.to_string();
//!         let token = token.clone();
//!         async move {
//!             // Simulate network request
//!             // …
//!         }
//!     }).detach();
//!
//!     signal
//! }
//! ```
//!
//! For complete usage examples, see the `examples/` directory in the repository.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]

// #[cfg(any(
//     all(feature = "gpui", feature = "dioxus"),
//     all(feature = "gpui", feature = "leptos"),
//     all(feature = "gpui", feature = "xilem"),
//     all(feature = "gpui", feature = "floem"),
//     all(feature = "dioxus", feature = "leptos"),
//     all(feature = "dioxus", feature = "xilem"),
//     all(feature = "dioxus", feature = "floem"),
//     all(feature = "leptos", feature = "xilem"),
//     all(feature = "leptos", feature = "floem"),
//     all(feature = "xilem", feature = "floem"),
// ))]
// compile_error!("Only one framework adapter feature may be enabled at a time.");
pub mod cancellation;
pub mod executor;
pub mod macros;
pub mod reactive;
pub mod signal;

pub use cancellation::CancellationToken;
pub use executor::{Executor, JoinHandle};
pub use reactive::ReactiveContext;
pub use signal::Signal;
pub mod prelude;
