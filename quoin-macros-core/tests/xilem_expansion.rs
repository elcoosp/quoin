//! Expansion-shape tests for the Xilem backend (the expansion gate).

#![cfg(feature = "xilem")]
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

#[test]
fn component_expands_to_struct_new_and_app_logic() {
    use quoin_macros_core::emit::render_xilem::component::emit_component;
    use quoin_macros_core::parse::ComponentAst;

    let ast: ComponentAst = syn::parse2(quote::quote! {
        pub CounterApp {
            state { count: u32 = 0 }
            render {
                quoin_render! {
                    div(class: "flex flex-col gap-2 p-4") {
                        h1() { "Counter" }
                        button(label: "Increment", on_click: |_| count.update(|c| *c += 1))
                    }
                }
            }
        }
    })
    .expect("component AST should parse");

    // A `component!` expansion is a set of items (struct + impl), not an
    // expression.
    let expanded = emit_component(&ast);
    let file: syn::File =
        syn::parse2(expanded.clone()).expect("component expansion must parse as items");
    assert_eq!(
        file.items.len(),
        2,
        "struct + impl, nothing else; got {:#?}",
        file.items
            .iter()
            .map(|i| match i {
                syn::Item::Struct(_) => "struct",
                syn::Item::Impl(_) => "impl",
                _ => "other",
            })
            .collect::<Vec<_>>()
    );

    let text = expanded.to_string();
    assert!(
        text.contains("struct CounterApp") && text.contains("XilemSignal < u32 >"),
        "one signal field per state declaration; expansion:\n{text}"
    );
    assert!(
        text.contains("fn new") && text.contains("ctx . create_signal (0)"),
        "new creates the signals from a context:\n{text}"
    );
    assert!(
        text.contains("fn app_logic") && text.contains("view :: flex"),
        "app_logic renders the tree:\n{text}"
    );
    assert!(
        text.contains("pump_view :: < CounterApp > ()"),
        "the pump view rides along in the tree:\n{text}"
    );
}

#[test]
fn component_with_keys_is_a_named_diagnostic() {
    use quoin_macros_core::emit::render_xilem::component::emit_component;
    use quoin_macros_core::parse::ComponentAst;

    let ast: ComponentAst = syn::parse2(quote::quote! {
        pub KeysApp {
            state { n: u32 = 0 }
            keys { "escape" => n.set(0) }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    })
    .expect("component AST should parse");
    let text = emit_component(&ast).to_string();
    assert!(
        text.contains("compile_error") && text.contains("keys"),
        "keys on Xilem must fail with the documented diagnostic:\n{text}"
    );
}

#[test]
fn run_app_expands_to_a_main_installing_the_pump() {
    use quoin_macros_core::emit::render_xilem::component::emit_run_app;
    use quoin_macros_core::run_app::RunAppInput;

    let input: RunAppInput =
        syn::parse2(quote::quote!(CounterApp)).expect("run_app AST should parse");
    let expanded = emit_run_app(&input);
    syn::parse2::<syn::Item>(expanded.clone())
        .expect("run_app expansion must parse as an item (one fn)");

    let text = expanded.to_string();
    assert!(
        text.contains("install_notifier"),
        "run_app! must install the rebuild pump notifier:\n{text}"
    );
    assert!(
        text.contains("new_simple_with_tokio"),
        "one tokio runtime serves the context and the driver:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// `effect!`: the core-runtime lowering (render-layer effect parity, M1.7)
// ---------------------------------------------------------------------------

fn emit_effect(source: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    use quoin_macros_core::effect::Effect;
    use quoin_macros_core::emit::{Emitter, FrameworkEmitter};

    let eff: Effect = syn::parse2(source).expect("effect AST should parse");
    Emitter.emit_effect(&eff)
}

#[test]
fn effect_expands_to_the_core_runtime_without_diagnostic() {
    let expanded = emit_effect(quote::quote! {
        deps: [count], run: || log(count.get())
    });
    let text = expanded.to_string();

    syn::parse2::<syn::Stmt>(expanded.clone())
        .expect("effect! expansion must parse as a statement");
    assert!(
        !text.contains("compile_error"),
        "effect! is native on Xilem now; no diagnostic:\n{text}"
    );
    assert!(
        text.contains(":: quoin :: xilem_effect :: create_effect"),
        "the effect must run on the core runtime through the adapter module:\n{text}"
    );
    assert!(
        text.contains("log (count . get ())"),
        "the run closure must be invoked, not evaluated and dropped:\n{text}"
    );
}

#[test]
fn effect_with_cleanup_expands_to_the_armed_cleanup_slot() {
    let expanded = emit_effect(quote::quote! {
        deps: [query],
        run: || fetch(query.get()),
        cleanup: || cancel()
    });
    let text = expanded.to_string();

    syn::parse2::<syn::Stmt>(expanded.clone())
        .expect("effect! expansion must parse as a statement");
    assert!(
        text.contains(":: quoin :: xilem_effect :: CleanupSlot :: new ()"),
        "cleanup lives in the shared Send+Sync slot:\n{text}"
    );
    assert!(
        text.contains("__quoin_armed"),
        "the initial run has nothing to tear down; an armed flag gates the cleanup:\n{text}"
    );
    // Ordering inside the closure: cleanup, then the body.
    let slot_run = text
        .find("__quoin_slot . run ()")
        .expect("the re-run cleanup must call slot.run");
    let body = text.find("fetch (query . get ())").expect("body invoked");
    assert!(
        slot_run < body,
        "cleanup must run before the body on re-runs:\n{text}"
    );
}

#[test]
fn component_mounts_effects_into_a_component_scope() {
    use quoin_macros_core::emit::render_xilem::component::emit_component;
    use quoin_macros_core::parse::ComponentAst;

    let ast: ComponentAst = syn::parse2(quote::quote! {
        pub EffectsApp {
            state { n: u32 = 0 }
            on_mount {
                effect! { deps: [n], run: || trace(n.get()) }
            }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    })
    .expect("component AST should parse");

    let expanded = emit_component(&ast);
    let text = expanded.to_string();
    assert!(
        !text.contains("compile_error"),
        "effect! in on_mount is the supported placement:\n{text}"
    );
    assert!(
        text.contains("__quoin_effects"),
        "the struct owns the component effect scope:\n{text}"
    );
    assert!(
        text.contains(":: quoin :: xilem_effect :: enter_effect_scope"),
        "on_mount runs inside the component scope so effect! registers into it:\n{text}"
    );
    assert!(
        text.contains("__quoin_scope_guard . pop ()"),
        "the scope guard is popped once on_mount is done:\n{text}"
    );
}

#[test]
fn effect_in_render_body_is_a_named_diagnostic() {
    use quoin_macros_core::emit::render_xilem::component::emit_component;
    use quoin_macros_core::parse::ComponentAst;

    let ast: ComponentAst = syn::parse2(quote::quote! {
        pub RenderEffectsApp {
            state { n: u32 = 0 }
            render {
                quoin_render! {
                    p() { "x" }
                    effect! { deps: [n], run: || trace(n.get()) }
                }
            }
        }
    })
    .expect("component AST should parse");

    let text = emit_component(&ast).to_string();
    assert!(
        text.contains("compile_error") && text.contains("on_mount"),
        "effect! in the render body would re-create the effect per rebuild; the \
         diagnostic must name the placement contract:\n{text}"
    );
}

#[test]
fn render_body_scan_ignores_non_macro_effect_mentions() {
    use quoin_macros_core::emit::render_xilem::component::emit_component;
    use quoin_macros_core::parse::ComponentAst;

    let ast: ComponentAst = syn::parse2(quote::quote! {
        pub ScanBaitApp {
            state { n: u32 = 0 }
            render {
                quoin_render! {
                    p() { { let effect = n.get(); let other = effect != 0; format!("{other}") } }
                }
            }
        }
    })
    .expect("component AST should parse");

    let text = emit_component(&ast).to_string();
    assert!(
        !text.contains("compile_error"),
        "a local named `effect` or a comparison must not trip the diagnostic:\n{text}"
    );
}
