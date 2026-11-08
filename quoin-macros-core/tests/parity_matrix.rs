//! Widget parity matrix — the machine-checked half of
//! `docs/BACKEND-PARITY.md`.
//!
//! Every element on the v1 path must lower to a native view on both dynamic
//! backends, or carry an [`ParityLevel::Unsupported`] row in the shared
//! `emit::parity` table whose diagnostic fires for real. The test reads the
//! table — there is no parallel carve-out list — and cross-checks it in
//! both directions:
//!
//! - a row claiming native/partial must **not** emit `compile_error!`;
//! - an emitter that **does** emit `compile_error!` for a known element must
//!   have an `Unsupported` row (never a silent drop, never an undocumented
//!   diagnostic);
//! - `docs/BACKEND-PARITY.md` must mention every row and its status.
//!
//! The matrix documents the **generic** lowerings — the default contract on
//! a no-shadcn build. The `<framework>-shadcn` features are an additive
//! visual layer with its own expansion coverage (the `quoin-floem`
//! conformance suite runs against the shadcn components when its `shadcn`
//! feature is on), so the matrix steps aside when `floem-shadcn` is
//! enabled: the two contracts legitimately differ in what they accept.

#![cfg(all(feature = "floem", feature = "xilem", not(feature = "floem-shadcn")))]
use quoin_macros_core::emit::parity::{BACKEND_PARITY, Back, ElementParity, ParityLevel, row};
use quoin_macros_core::emit::{render_floem, render_xilem};
use quoin_macros_core::render_ast::RenderNode;
use syn::parse_quote;

/// A minimal, well-formed source snippet per element — just enough for the
/// emitter to reach its element dispatch (arg-level errors like a
/// `dropdown_menu` without items are out of scope here).
fn minimal_source(element: &str) -> Option<RenderNode> {
    let source: proc_macro2::TokenStream = match element {
        "checkbox" => parse_quote! { checkbox(checked: true) { "Ship it" } },
        "switch" => parse_quote! { switch(checked: false) },
        "slider" => parse_quote! { slider(value: 0.5, min: 0.0, max: 1.0) },
        "progress" => parse_quote! { progress(value: 0.75) },
        "scroll_area" => parse_quote! { scroll_area(class: "h-64") { div() { "content" } } },
        "tabs" => parse_quote! {
            tabs(active: 0, on_click: move |i| active.set(i)) {
                tab(index: 0, label: "One") { "first" }
                tab(index: 1, label: "Two") { "second" }
            }
        },
        "data_table" => parse_quote! {
            data_table(rows: rows) {
                column(key: "name", label: "Name", render: |row: &Row| row.name.clone())
            }
        },
        "virtual_list" => parse_quote! {
            virtual_list(items: rows, estimated_height: 32.0) { p() { "x" } }
        },
        "dropdown_menu" => parse_quote! {
            dropdown_menu(label: "Menu") {
                item(label: "First", on_click: |_| {})
            }
        },
        "clipboard_button" => parse_quote! { clipboard_button(copy_text: "hi") },
        "tooltip" => parse_quote! { tooltip(text: "hint") { div() { "target" } } },
        "icon" => parse_quote! { icon(icon_name: "check") },
        "img" => parse_quote! { img(src: "x.png") },
        "svg" => parse_quote! { svg(src: "<svg/>") },
        "rich_text" => parse_quote! { rich_text(text: "hi") },
        "styled_text" => parse_quote! { styled_text(text: "hi", runs: Vec::new()) },
        "radio" => parse_quote! { radio(checked: false, value: "a") },
        "radio_group" => parse_quote! { radio_group() { radio(checked: false, value: "a") } },
        "skeleton_text" => parse_quote! { skeleton_text() },
        "skeleton_avatar" => parse_quote! { skeleton_avatar() },
        // Plain containers and text elements take a default child.
        other => {
            let ident = proc_macro2::Ident::new(other, proc_macro2::Span::call_site());
            parse_quote! { #ident() { "x" } }
        }
    };
    syn::parse2(source).ok()
}

fn floem_text(node: &RenderNode) -> String {
    render_floem::emit_render(node).to_string()
}

fn xilem_text(node: &RenderNode) -> String {
    render_xilem::emit_render(node).to_string()
}

fn expansion_for(backend: Back, node: &RenderNode) -> String {
    match backend {
        Back::Floem => floem_text(node),
        Back::Xilem => xilem_text(node),
    }
}

fn level_for(row: &ElementParity, backend: Back) -> ParityLevel {
    match backend {
        Back::Floem => row.floem,
        Back::Xilem => row.xilem,
    }
}

/// A row claiming native/partial must expand without a diagnostic, and the
/// expansion must parse as an expression.
#[test]
fn native_rows_emit_no_diagnostic() {
    for row in BACKEND_PARITY {
        for backend in [Back::Floem, Back::Xilem] {
            let level = level_for(row, backend);
            if matches!(level, ParityLevel::Unsupported(_)) {
                continue;
            }
            let Some(node) = minimal_source(row.element) else {
                panic!("no minimal source for `{}`", row.element);
            };
            let expansion = expansion_for(backend, &node);
            assert!(
                !expansion.contains("compile_error") && !expansion.contains("not supported"),
                "`{}` is {:?} on {backend:?} but the expansion has a diagnostic:\n{expansion}",
                row.element,
                level
            );
            syn::parse_str::<syn::Expr>(&expansion).unwrap_or_else(|e| {
                panic!("{backend:?} expansion of `{}` must parse: {e}", row.element)
            });
        }
    }
}

/// An `Unsupported` row must really fire a named diagnostic.
#[test]
fn unsupported_rows_emit_named_diagnostics() {
    for row in BACKEND_PARITY {
        for backend in [Back::Floem, Back::Xilem] {
            let ParityLevel::Unsupported(reason) = level_for(row, backend) else {
                continue;
            };
            let Some(node) = minimal_source(row.element) else {
                panic!("no minimal source for `{}`", row.element);
            };
            let expansion = expansion_for(backend, &node);
            assert!(
                expansion.contains("compile_error") && expansion.contains(row.element),
                "`{}` is documented unsupported on {backend:?} ({reason}) but the \
                 expansion does not carry a diagnostic naming it:\n{expansion}",
                row.element
            );
        }
    }
}

/// The inverse direction: any emitter diagnostic for a known element must
/// have a matching `Unsupported` row in the table — no undocumented
/// rejections, no silent degradations sneaking past the doc.
#[test]
fn emitter_diagnostics_are_all_documented() {
    for element in KNOWN_V1_ELEMENTS {
        let Some(node) = minimal_source(element) else {
            panic!("no minimal source for `{element}`");
        };
        for backend in [Back::Floem, Back::Xilem] {
            let expansion = expansion_for(backend, &node);
            let rejects = expansion.contains("compile_error");
            let documented = row(element)
                .is_some_and(|r| matches!(level_for(r, backend), ParityLevel::Unsupported(_)));
            assert_eq!(
                rejects,
                documented,
                "`{element}` on {backend:?}: emitter {} a diagnostic but the parity table {} one \
                 (expansion:\n{expansion})",
                if rejects { "emits" } else { "does not emit" },
                if documented { "has" } else { "has no" }
            );
        }
    }
}

/// `docs/BACKEND-PARITY.md` must mention every row, its Floem status and its
/// Xilem status, so the markdown cannot drift from the machine-checked
/// table.
#[test]
fn parity_doc_matches_table() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let doc =
        std::fs::read_to_string(std::path::Path::new(manifest).join("../docs/BACKEND-PARITY.md"))
            .expect("docs/BACKEND-PARITY.md must exist next to the workspace");
    for row in BACKEND_PARITY {
        assert!(
            doc.contains(row.element),
            "docs/BACKEND-PARITY.md does not mention `{}`",
            row.element
        );
        for (backend, level) in [("floem", row.floem), ("xilem", row.xilem)] {
            let token = match level {
                ParityLevel::Native => format!("`{backend}: native`"),
                ParityLevel::Partial(_) => format!("`{backend}: partial`"),
                ParityLevel::Unsupported(_) => format!("`{backend}: unsupported`"),
            };
            assert!(
                doc.contains(&token),
                "docs/BACKEND-PARITY.md must mark `{}` with `{token}`",
                row.element
            );
        }
    }
}

/// Every element of the v1 path the matrix covers. Structural children
/// (`tab`, `column`, `item`) and the plain HTML-ish containers are lowered
/// generically on both dynamic backends and have no dedicated rows.
const KNOWN_V1_ELEMENTS: &[&str] = &[
    "button",
    "input",
    "checkbox",
    "switch",
    "slider",
    "progress",
    "badge",
    "skeleton",
    "skeleton_text",
    "skeleton_avatar",
    "separator",
    "scroll_area",
    "virtual_list",
    "tabs",
    "data_table",
    "dropdown_menu",
    "clipboard_button",
    "tooltip",
    "icon",
    "img",
    "svg",
    "rich_text",
    "styled_text",
    "radio",
    "radio_group",
];

/// Every matrix row must be reachable from `KNOWN_V1_ELEMENTS` (and vice
/// versa) so the two lists cannot drift.
#[test]
fn table_covers_exactly_the_v1_elements() {
    let mut table_elements: Vec<&str> = BACKEND_PARITY.iter().map(|r| r.element).collect();
    table_elements.sort_unstable();
    let mut known: Vec<&str> = KNOWN_V1_ELEMENTS.to_vec();
    known.sort_unstable();
    assert_eq!(table_elements, known, "table and v1 element list diverged");
}
