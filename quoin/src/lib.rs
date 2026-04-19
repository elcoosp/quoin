//! quoin - One reactive core, many frameworks.
//!
//! This crate provides foundational traits for building framework-agnostic
//! reactive libraries in Rust. Enable exactly one adapter feature to select
//! your target UI framework.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all, clippy::pedantic)]

#[cfg(any(
    all(feature = "gpui", feature = "dioxus"),
    all(feature = "gpui", feature = "leptos"),
    all(feature = "gpui", feature = "xilem"),
    all(feature = "gpui", feature = "floem"),
    all(feature = "dioxus", feature = "leptos"),
    all(feature = "dioxus", feature = "xilem"),
    all(feature = "dioxus", feature = "floem"),
    all(feature = "leptos", feature = "xilem"),
    all(feature = "leptos", feature = "floem"),
    all(feature = "xilem", feature = "floem"),
))]
compile_error!("Only one framework adapter feature may be enabled at a time.");

pub mod cancellation;
pub mod executor;
pub mod reactive;
pub mod signal;

pub use cancellation::CancellationToken;
pub use executor::{Executor, JoinHandle};
pub use reactive::ReactiveContext;
pub use signal::Signal;
