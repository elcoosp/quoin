//! Expansion-shape tests for the Xilem backend (the expansion gate).

use quoin_macros_core::emit::render_xilem::emit_render;
use quoin_macros_core::render_ast::RenderNode;
use syn::parse_quote;

fn expand(source: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let node: RenderNode = syn::parse2(source).expect("render AST should parse");
    emit_render(&node)
}

#[test]
fn static_counter_tree_expands_without_boxing() {
    let source = parse_quote! {
        div(class: "flex flex-col gap-2 p-4") {
            h1() { "Counter" }
            p() { { format!("Count: {}", counter.count.get()) } }
            button(label: "+", on_click: |_| counter.count.update(|c| *c += 1))
        }
    };
    let expanded = expand(source);
    let text = expanded.to_string();

    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");

    for banned in ["into_any", ". boxed ()", "Rc ::", "Arc ::"] {
        assert!(
            !text.contains(banned),
            "fully static tree must not contain `{banned}`; expansion:\n{text}"
        );
    }
    // Static children use the native flex tuple.
    assert!(
        text.contains(":: xilem :: view :: flex"),
        "expected flex in:\n{text}"
    );
    // The dynamic text is evaluated eagerly (no closure), per the plan.
    assert!(
        !text.contains("Label :: derived"),
        "xilem labels take the value directly:\n{text}"
    );
    // Handlers are moved directly: no Rc/Arc wrapper.
    assert!(
        !text.contains("Rc :: new") && !text.contains("Arc :: new"),
        "handlers must not be wrapped:\n{text}"
    );
}

#[test]
fn if_else_boxes_exactly_once_per_branch() {
    let source = parse_quote! {
        div() {
            if[visible] {
                div() { "On" }
            } else {
                div() { "Off" }
            }
        }
    };
    let expanded = expand(source);
    let text = expanded.to_string();
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert_eq!(
        text.matches(". boxed ()").count(),
        2,
        "one seam = one box per branch; expansion:\n{text}"
    );
}

#[test]
fn if_without_else_uses_option_child() {
    let source = parse_quote! {
        div() {
            if[visible] {
                div() { "On" }
            }
        }
    };
    let expanded = expand(source);
    let text = expanded.to_string();
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert!(
        !text.contains(". boxed ()"),
        "if without else must not box:\n{text}"
    );
    assert!(text.contains(". then ("), "expected Option::then:\n{text}");
}

#[test]
fn for_expands_to_index_keyed_vec_sequence() {
    let source = parse_quote! {
        div() {
            for[item in items] {
                p() { { item.name.clone() } }
            }
        }
    };
    let expanded = expand(source);
    let text = expanded.to_string();
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert!(
        text.contains(". into_iter () . map ("),
        "for must expand to a Vec view sequence:\n{text}"
    );
}

#[test]
fn text_style_cascades_to_label_leaves() {
    let source = parse_quote! {
        div(class: "text-muted-foreground text-lg") {
            p() { "hello" }
        }
    };
    let expanded = expand(source);
    let text = expanded.to_string();
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert!(
        text.contains(". color ("),
        "container text color must cascade to the leaf:\n{text}"
    );
    assert!(
        text.contains(". text_size ("),
        "container text size must cascade to the leaf:\n{text}"
    );
    // The muted-foreground token resolves through the adapter theme.
    assert!(
        text.contains(":: quoin :: xilem_theme :: muted_foreground_text ()"),
        "expected the theme token call:\n{text}"
    );
}

#[test]
fn unknown_class_is_a_compile_error() {
    let source = parse_quote! {
        div(class: "definitely-not-a-class") { "x" }
    };
    let text = expand(source).to_string();
    assert!(
        text.contains("compile_error") && text.contains("definitely-not-a-class"),
        "unknown classes must fail with a diagnostic:\n{text}"
    );
}
