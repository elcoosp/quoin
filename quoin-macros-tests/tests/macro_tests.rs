//! Macro UI tests (trybuild): `component!`, `quoin_render!`, `effect!`, and
//! `run_app!` expansions must compile for the enabled framework and produce
//! the expected diagnostics for invalid input.
//!
//! Run one framework at a time (mirrors CI):
//!
//! ```bash
//! cargo test -p quoin-macros-tests --features gpui
//! cargo test -p quoin-macros-tests --features leptos
//! cargo test -p quoin-macros-tests --features dioxus
//! ```
//!
//! Feature propagation: trybuild builds its scratch project as a path
//! dependency on this crate, inheriting the feature resolution of the
//! surrounding `cargo test` invocation, so no manual environment plumbing is
//! needed here.

#[cfg(test)]
mod tests {
    #[cfg(feature = "gpui")]
    #[test]
    fn ui_tests_gpui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_gpui_pass.rs");
    }

    #[cfg(feature = "leptos")]
    #[test]
    fn ui_tests_leptos() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_leptos_pass.rs");
    }

    #[cfg(feature = "dioxus")]
    #[test]
    fn ui_tests_dioxus() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_dioxus_pass.rs");
    }
}
