//! Common imports for the active framework.
//!
//! ```rust,ignore
//! use quoin::prelude::*;
//! ```
//!
//! Brings in core reactive traits, the active framework's adapter types,
//! and the declarative macros — all in one line.

// Core traits — always available
pub use quoin_core::{CancellationToken, Executor, ReactiveContext, Signal};

// Active framework adapter — exactly one at a time
#[cfg(feature = "gpui")]
pub use quoin_gpui::*;

#[cfg(feature = "dioxus")]
pub use quoin_dioxus::*;

#[cfg(feature = "leptos")]
pub use quoin_leptos::*;

#[cfg(feature = "floem")]
pub use quoin_floem::*;

#[cfg(feature = "xilem")]
pub use quoin_xilem::*;

// Macros — when any framework (or explicit macros feature) is active
#[cfg(any(feature = "gpui", feature = "leptos", feature = "dioxus", feature = "macros"))]
pub use quoin_macros::{component, effect, quoin_element, quoin_render, run_app};
