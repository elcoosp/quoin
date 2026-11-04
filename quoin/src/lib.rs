//! # quoin – One Reactive Core, Many Frameworks
//!
//! **quoin** is the facade crate for the quoin ecosystem. It re-exports the core
//! reactive traits from [`quoin_core`] and, when a framework feature is enabled,
//! also re-exports the corresponding adapter types and macros.
//!
//! ## Quick Start
//!
//! Add `quoin` to your `Cargo.toml` and enable **exactly one** adapter feature:
//!
//! ```toml
//! [dependencies]
//! quoin = { version = "0.1", features = ["gpui"] }
//! ```
//!
//! Then use the unified API:
//!
//! ```rust,ignore
//! use quoin::{ReactiveContext, Signal, GpuiContext, component, quoin_render};
//! ```
//!
//! All adapter types (`GpuiContext`, `LeptosContext`, …) and macros
//! (`component!`, `quoin_render!`, `effect!`) are available directly from
//! `quoin` when the corresponding feature is enabled.

// ── Compile-time safety: only one framework feature ─────────────────────────

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
compile_error!(
    "Only one framework adapter feature may be enabled at a time. \
     Choose exactly one: `gpui`, `dioxus`, `leptos`, `xilem`, or `floem`."
);

// ── Core re-exports (always available) ─────────────────────────────────────

pub use futures_core;
pub use quoin_core::*;

// ── GPUI adapter ───────────────────────────────────────────────────────────

#[cfg(feature = "gpui")]
pub use quoin_gpui::*;
#[cfg(feature = "gpui")]
pub use quoin_ui_gpui::*;

// ── Dioxus adapter ─────────────────────────────────────────────────────────

#[cfg(feature = "dioxus")]
pub use quoin_dioxus::*;

// ── Leptos adapter ─────────────────────────────────────────────────────────

#[cfg(feature = "leptos")]
pub use quoin_leptos::*;

// ── Xilem adapter ──────────────────────────────────────────────────────────

#[cfg(feature = "xilem")]
/// The Xilem rebuild pump (see `quoin-xilem/src/pump.rs`).
pub use quoin_xilem::pump as xilem_pump;
#[cfg(feature = "xilem")]
/// The Xilem theme module under an unambiguous name (the emitter's generated
/// code references `::quoin::xilem_theme::…`).
pub use quoin_xilem::theme as xilem_theme;
#[cfg(feature = "xilem")]
pub use quoin_xilem::*;

// ── Floem adapter ──────────────────────────────────────────────────────────

#[cfg(feature = "floem")]
/// The Floem effect-cleanup slot under an unambiguous name: the emitter's
/// `effect!` expansion references `::quoin::floem_effect::…`.
pub use quoin_floem::effect as floem_effect;
#[cfg(feature = "floem")]
/// The Floem keystroke matcher under an unambiguous name: the emitter's
/// `keys { .. }` expansion references `::quoin::floem_keys::…`.
pub use quoin_floem::keys as floem_keys;
#[cfg(feature = "floem")]
/// The Floem theme module under an unambiguous name: the glob re-export
/// above already puts its tokens at the facade root, and the emitter's
/// generated code references `::quoin::floem_theme::…`.
pub use quoin_floem::theme as floem_theme;
#[cfg(feature = "floem")]
pub use quoin_floem::*;

// ── Macros (re-exported when any framework or explicit macros feature) ─────

#[cfg(any(
    feature = "gpui",
    feature = "dioxus",
    feature = "leptos",
    feature = "xilem",
    feature = "floem",
    feature = "macros",
))]
pub use quoin_macros::{component, effect, quoin_element, quoin_render, run_app};

// ── UCP traits ─────────────────────────────────────────────────────────────

#[cfg(feature = "ui")]
pub use quoin_ui::*;

// ── Testing & devtools ─────────────────────────────────────────────────────

#[cfg(feature = "testkit")]
pub use quoin_testkit as testkit;

#[cfg(feature = "devtools")]
pub use quoin_devtools as devtools;

pub mod prelude;
