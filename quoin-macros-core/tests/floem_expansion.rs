//! Expansion-shape tests for the Floem backend.
//!
//! These are the kernel of the CI **expansion gate** (plan section 3): the
//! tokens a fully static tree produces must parse as an expression, must not
//! contain any dynamic dispatch/boxing (`into_any`, `boxed`, `dyn`), and must
//! not allocate (`Rc ::`, `Arc ::`, `Vec`). Dynamic seams are allowed exactly
//! one seam per `if`/`for`.

use quoin_macros_core::emit::render_floem::emit_render;
use quoin_macros_core::render_ast::RenderNode;
use syn::parse_quote;

fn expand(source: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let node: RenderNode = syn::parse2(source).expect("render AST should parse");
    emit_render(&node)
}

fn tokens_to_string(tokens: &proc_macro2::TokenStream) -> String {
    tokens.to_string()
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
    let text = tokens_to_string(&expanded);

    // The expansion must parse as an expression — the compiler-level check
    // for the whole backend's token shapes.
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");

    for banned in ["into_any", "boxed", "dyn ", "Rc ::", "Arc ::", "Vec"] {
        assert!(
            !text.contains(banned),
            "fully static tree must not contain `{banned}`; expansion:\n{text}"
        );
    }

    // Static children use the native tuple constructor.
    assert!(
        text.contains("Stack :: vertical"),
        "expected Stack::vertical in:\n{text}"
    );
    // Text lowers to label; the static literal stays closure-wrapped.
    assert!(
        text.contains("Label :: derived"),
        "expected Label::derived in:\n{text}"
    );
    // The dynamic text is shadow-cloned then moved: the plan's expected shape.
    assert!(
        text.contains(":: core :: clone :: Clone :: clone (& counter)"),
        "expected a shadow clone of `counter`; expansion:\n{text}"
    );
    // No runtime class parsing survives.
    assert!(
        !text.contains("flex flex-col"),
        "class strings must not survive expansion:\n{text}"
    );
}

#[test]
fn if_else_expands_to_exactly_one_seam() {
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
    let text = tokens_to_string(&expanded);
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert_eq!(
        text.matches("into_any").count(),
        2,
        "one seam = two into_any (then + else); expansion:\n{text}"
    );
    assert!(
        text.contains("dyn_container"),
        "expected dyn_container:\n{text}"
    );
}

#[test]
fn keyed_for_expands_to_dyn_stack() {
    let source = parse_quote! {
        div() {
            for[item in items, key: item.id] {
                p() { { item.name.clone() } }
            }
        }
    };
    let expanded = expand(source);
    let text = tokens_to_string(&expanded);
    syn::parse2::<syn::Expr>(expanded).expect("expansion must be a valid expression");
    assert!(
        text.contains("dyn_stack"),
        "keyed for must use dyn_stack:\n{text}"
    );
}

#[test]
fn unkeyed_for_is_a_compile_error() {
    let source = parse_quote! {
        div() {
            for[item in items] {
                p() { { item.name.clone() } }
            }
        }
    };
    let text = tokens_to_string(&expand(source));
    assert!(
        text.contains("compile_error"),
        "unkeyed for must emit the key diagnostic:\n{text}"
    );
}

#[test]
fn unknown_class_is_a_compile_error() {
    let source = parse_quote! {
        div(class: "definitely-not-a-class") { "x" }
    };
    let text = tokens_to_string(&expand(source));
    assert!(
        text.contains("compile_error") && text.contains("definitely-not-a-class"),
        "unknown classes must fail with a diagnostic:\n{text}"
    );
}

#[test]
fn unknown_element_is_rejected_at_parse() {
    let source: proc_macro2::TokenStream = parse_quote! {
        nosuchelement() { "x" }
    };
    let result = syn::parse2::<RenderNode>(source);
    assert!(result.is_err(), "unknown elements must fail to parse");
    let message = result.err().unwrap().to_string();
    assert!(message.contains("unknown element"), "diagnostic: {message}");
}
