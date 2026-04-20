use trybuild::TestCases;

#[cfg(feature = "gpui")]
#[test]
fn ui_tests_gpui() {
    let t = TestCases::new();
    t.pass("tests/ui/render_gpui_pass.rs");
    t.compile_fail("tests/ui/component_missing_render.rs");
    t.compile_fail("tests/ui/render_missing_colon.rs");
}
