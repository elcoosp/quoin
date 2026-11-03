//! Semantic-probe golden file (rendering-parity plan, layer 1).
//!
//! Every backend lowers the same Tailwind class strings through the same
//! shared IR (`parse_tailwind` → `Vec<Decl>`), so the framework-independent
//! description of each class — `Decl::describe()` — must be stable. The
//! committed `.golden` file is the semantic contract: if a change to the IR
//! or the vocabulary alters the resolved style of any class, this test fails
//! and the golden file must be re-reviewed and regenerated intentionally
//! (`QUOIN_UPDATE_GOLDEN=1 cargo test -p quoin-style --test semantic_golden`).
//!
//! This is the probe half of the rendering-consistency verification: the
//! backends are only required to agree because they all consume these exact
//! `Decl` values, and this test pins the values down.

/// The canonical class set every backend's docs and examples rely on.
/// Groups mirror the plan's probe fixtures (counter, todo, devtools panel).
const CANONICAL_CLASSES: &[(&str, &[&str])] = &[
    (
        "counter-app",
        &[
            "flex flex-col gap-2 p-4",
            "text-2xl font-bold",
            "text-muted-foreground text-sm",
            "px-3 py-1 rounded",
            "w-full h-11",
        ],
    ),
    (
        "todo-app",
        &[
            "flex flex-row items-center gap-3",
            "flex-1 min-w-0",
            "text-gray-400 text-sm",
            "border border-gray-200 rounded-md",
            "size-full overflow-y-scroll",
        ],
    ),
    (
        "devtools-panel",
        &[
            "absolute bg-card text-card",
            "hidden truncate select-none",
            "bg-muted text-muted-foreground",
            "hover:bg-popover",
            "border-b border-gray-700 rounded-none",
        ],
    ),
];

/// Renders the semantic dump for the canonical classes.
fn semantic_dump() -> String {
    let mut out = String::new();
    for (group, classes) in CANONICAL_CLASSES {
        out.push_str("# ");
        out.push_str(group);
        out.push('\n');
        for class in *classes {
            let parsed = quoin_style::parse_tailwind(class);
            out.push_str("class: ");
            out.push_str(class);
            out.push('\n');
            for decl in &parsed.base {
                out.push_str("  base:   ");
                out.push_str(&decl.describe());
                out.push('\n');
            }
            for decl in &parsed.hover {
                out.push_str("  hover:  ");
                out.push_str(&decl.describe());
                out.push('\n');
            }
            assert!(
                parsed.unknown.is_empty(),
                "canonical class `{class}` must stay inside the known vocabulary: {:?}",
                parsed.unknown
            );
        }
    }
    out
}

const GOLDEN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/semantic_probe.golden");

#[test]
fn canonical_classes_describe_stably() {
    let actual = semantic_dump();
    if let Ok(expected) = std::fs::read_to_string(GOLDEN_PATH) {
        assert_eq!(
            actual, expected,
            "the semantic probe changed: re-review the diff, then regenerate with \
             QUOIN_UPDATE_GOLDEN=1"
        );
    } else {
        std::fs::write(GOLDEN_PATH, &actual).expect("write golden file");
        panic!("golden file was missing; wrote a new one — rerun to confirm");
    }
}

#[test]
fn golden_regeneration_is_explicit() {
    // `QUOIN_UPDATE_GOLDEN=1` regenerates instead of failing — the explicit
    // human step the plan requires before a golden change lands.
    if std::env::var("QUOIN_UPDATE_GOLDEN").is_err() {
        return;
    }
    std::fs::write(GOLDEN_PATH, semantic_dump()).expect("write golden file");
}
