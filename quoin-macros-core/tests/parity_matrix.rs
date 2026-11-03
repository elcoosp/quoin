//! Widget parity matrix (M5 exit criterion): every element on the v1 parity
//! path must lower to a native view on both dynamic backends — never to an
//! `unsupported_element` / `compile_error!` diagnostic.
//!
//! GPUI/Leptos/Dioxus are the pre-existing reference backends; Floem and
//! Xilem joined in M2/M3, so the matrix here guards their coverage.

use quoin_macros_core::emit::render_floem;
use quoin_macros_core::emit::render_xilem;
use quoin_macros_core::render_ast::RenderNode;
use syn::parse_quote;

/// The M5 parity set. Each entry is the element with its minimal
/// framework-agnostic arguments.
fn parity_elements() -> Vec<RenderNode> {
    vec![
        parse_quote! { checkbox(checked: true) { "Ship it" } },
        parse_quote! { switch(checked: false) },
        parse_quote! { slider(value: 0.5, min: 0.0, max: 1.0) },
        parse_quote! { scroll_area(class: "h-64") { div() { "content" } } },
        parse_quote! {
            tabs(tabs: vec!["One".to_string(), "Two".to_string()], active: 0,
                 on_click: move |i| active.set(i))
        },
        parse_quote! {
            virtual_list(items: rows, estimated_height: 32.0) {
                p() { { item.clone() } }
            }
        },
        parse_quote! {
            dropdown_menu(label: "Menu") {
                item(label: "First", on_click: |_| {})
                item(label: "Second", on_click: |_| {})
            }
        },
        parse_quote! { badge(text: "beta") },
        parse_quote! { progress(value: 0.75) },
        parse_quote! { skeleton() },
        parse_quote! { separator() },
    ]
}

/// Elements with an explicit per-backend diagnostic contract (never silent):
/// - `tooltip`: native on Floem; Xilem has no tooltip view yet, so it must
///   fail with a named `compile_error!` until Masonry grows one (plan §M5).
fn diagnostic_contract_elements() -> Vec<RenderNode> {
    vec![parse_quote! { tooltip(text: "hint") { div() { "target" } } }]
}

fn floem_text(node: &RenderNode) -> String {
    render_floem::emit_render(node).to_string()
}

fn xilem_text(node: &RenderNode) -> String {
    render_xilem::emit_render(node).to_string()
}

#[test]
fn parity_matrix_has_no_unsupported_elements() {
    let nodes = parity_elements();
    for node in &nodes {
        let name = match node {
            RenderNode::Element(el) => el.name.to_string(),
            _ => String::new(),
        };
        let floem = floem_text(node);
        assert!(
            !floem.contains("compile_error") && !floem.contains("not supported"),
            "Floem parity violation for `{name}`:\n{floem}"
        );
        let xilem = xilem_text(node);
        assert!(
            !xilem.contains("compile_error") && !xilem.contains("not supported"),
            "Xilem parity violation for `{name}`:\n{xilem}"
        );
    }
}

#[test]
fn diagnostic_contract_elements_fail_loudly() {
    for node in &diagnostic_contract_elements() {
        let name = match node {
            RenderNode::Element(el) => el.name.to_string(),
            _ => String::new(),
        };
        // Floem lowers it natively.
        let floem = floem_text(node);
        assert!(
            !floem.contains("compile_error"),
            "Floem must lower `{name}` natively:\n{floem}"
        );
        // Xilem's diagnostic must name the element — silence is never an option.
        let xilem = xilem_text(node);
        assert!(
            xilem.contains("compile_error") && xilem.contains(&name),
            "Xilem must reject `{name}` with a named diagnostic:\n{xilem}"
        );
    }
}

#[test]
fn parity_matrix_output_parses_on_both_backends() {
    for node in &parity_elements() {
        syn::parse2::<syn::Expr>(render_floem::emit_render(node))
            .unwrap_or_else(|e| panic!("floem expansion must parse: {e}"));
        syn::parse2::<syn::Expr>(render_xilem::emit_render(node))
            .unwrap_or_else(|e| panic!("xilem expansion must parse: {e}"));
    }
}
