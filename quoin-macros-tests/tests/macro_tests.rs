#[cfg(test)]
mod tests {
    #[test]
    fn ui_tests_gpui() {
        #[cfg(feature = "gpui")]
        {
            let t = trybuild::TestCases::new(); // Tells trybuild to use this package's Cargo.toml

            t.compile_fail("tests/ui/*_fail.rs");
            t.pass("tests/ui/*_gpui_pass.rs");
        }
    }

    #[test]
    fn ui_tests_leptos() {
        #[cfg(feature = "leptos")]
        {
            let t = trybuild::TestCases::new();

            t.compile_fail("tests/ui/*_fail.rs");
            t.pass("tests/ui/*_leptos_pass.rs");
        }
    }

    #[test]
    fn ui_tests_dioxus() {
        #[cfg(feature = "dioxus")]
        {
            let t = trybuild::TestCases::new();

            t.compile_fail("tests/ui/*_fail.rs");
            t.pass("tests/ui/*_dioxus_pass.rs");
        }
    }
}
