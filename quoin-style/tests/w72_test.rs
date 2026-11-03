//! Unit tests for the w-72 width parsing path.

#[test]
fn spacing_widths_parse() {
    let parsed = quoin_style::parse_tailwind("w-72 w-full h-11");
    assert!(parsed.unknown.is_empty(), "{:?}", parsed.unknown);
    assert_eq!(parsed.base.len(), 3);
}
