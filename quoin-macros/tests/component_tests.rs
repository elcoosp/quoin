#[test]
fn component_macro_gpui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/component_gpui.rs");
}
