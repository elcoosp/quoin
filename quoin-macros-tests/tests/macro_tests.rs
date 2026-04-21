#[cfg(test)]
mod tests {
    use std::env;

    fn setup_trybuild_features() {
        let features_str = include_str!(concat!(env!("OUT_DIR"), "/features.txt"));

        if !features_str.is_empty() {
            let mut args = Vec::new();
            for feature in features_str.split(',') {
                args.push("--features".to_string());
                args.push(feature.to_string());
            }
            // trybuild will pick up this environment variable
            env::set_var("TRYBUILD_CARGO_OPTIONS", args.join(" "));
        }
    }

    #[cfg(feature = "gpui")]
    #[test]
    fn ui_tests_gpui() {
        setup_trybuild_features();
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_gpui_pass.rs");
    }

    #[cfg(feature = "leptos")]
    #[test]
    fn ui_tests_leptos() {
        setup_trybuild_features();
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_leptos_pass.rs");
    }

    #[cfg(feature = "dioxus")]
    #[test]
    fn ui_tests_dioxus() {
        setup_trybuild_features();
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*_fail.rs");
        t.pass("tests/ui/*_dioxus_pass.rs");
    }
}
