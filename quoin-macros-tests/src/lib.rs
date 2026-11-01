//! Compile-time fixtures for the `quoin-macros` UI (trybuild) test suite.
//!
//! This crate intentionally exposes no public API of its own: it is the
//! linkage surface that lets `quoin-macros-tests/tests/*.rs` expand macros
//! against the real framework backends (GPUI, Leptos, Dioxus) selected by
//! feature flags. The enabled features are captured by `build.rs` into an
//! include-able file so tests can assert on the active backend.
