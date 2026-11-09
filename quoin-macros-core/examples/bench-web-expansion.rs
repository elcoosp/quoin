//! Micro-benchmarks for the web-backend (`component!`) expansion path.
//!
//! The `keys { .. }` branch is a compile-time switch on `ComponentAst::keys`;
//! since the Phase 1.11 web-keys milestone the with-keys branch emits the
//! real lowering (shared keystroke parser, one window listener, one matcher
//! arm per entry) rather than the Phase 1.9 diagnostic. These benches pin
//! the expansion cost of both shapes: the keys-free path — every component
//! without shortcuts — is unchanged, and the with-keys delta is ~10 µs of
//! token assembly paid once per component at compile time (the runtime
//! matcher it emits costs 1–2 ns/op; see the `quoin-core` rows in
//! `docs/BENCHMARKS.md`).
//!
//! No benchmark framework — plain `std::time::Instant` so this runs anywhere:
//!
//! ```bash
//! just bench-web-expansion
//! # cargo run --release -p quoin-macros-core --features leptos,dioxus \
//! #   --example bench-web-expansion
//! ```
//!
//! Numbers are machine-relative; treat them as regression baselines to
//! compare across commits on the same machine, not absolute truth.

use quoin_macros_core::emit::dioxus::emit_component as emit_dioxus;
use quoin_macros_core::emit::leptos::emit_component as emit_leptos;
use quoin_macros_core::parse::ComponentAst;
use std::time::Instant;
use syn::parse_quote;

const ITERATIONS: u32 = 20_000;

/// Parses and emits one `component!` body per iteration.
fn bench<F: FnMut()>(name: &str, iterations: u32, mut f: F) {
    // Warmup.
    f();
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_secs_f64() * 1e9 / f64::from(iterations);
    println!("{name:<44} {iterations:>7} ops in {elapsed:>10.2?}  ({ns_per_op:>7.0} ns/op)");
}

fn main() {
    println!("quoin-macros-core web-expansion micro-benchmarks (release build)\n");

    let plain: proc_macro2::TokenStream = parse_quote! {
        pub Plain {
            state { n: u32 = 0 }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    };
    let keyed: proc_macro2::TokenStream = parse_quote! {
        pub Keyed {
            state { n: u32 = 0 }
            keys { "escape" => n.set(0) }
            render {
                quoin_render! { p() { "x" } }
            }
        }
    };

    // The diagnostic branch in isolation: emit-only over a pre-parsed AST,
    // so the delta is the `keys.is_empty()` branch plus the `quote!` block —
    // not the extra `keys { .. }` section the parser must read.
    let plain_ast: ComponentAst = syn::parse2(plain.clone()).expect("parses");
    let keyed_ast: ComponentAst = syn::parse2(keyed.clone()).expect("parses");
    bench("leptos emit-only (no keys)", ITERATIONS, || {
        let _ = emit_leptos(&plain_ast);
    });
    bench("leptos emit-only (keys lowering)", ITERATIONS, || {
        let _ = emit_leptos(&keyed_ast);
    });
    bench("dioxus emit-only (no keys)", ITERATIONS, || {
        let _ = emit_dioxus(&plain_ast);
    });
    bench("dioxus emit-only (keys lowering)", ITERATIONS, || {
        let _ = emit_dioxus(&keyed_ast);
    });

    // End to end (parse + emit): the with-keys delta here is dominated by
    // the extra `keys { .. }` section the parser reads, not by the
    // diagnostic.
    bench("leptos parse+emit (no keys)", ITERATIONS, || {
        let ast: ComponentAst = syn::parse2(plain.clone()).expect("parses");
        let _ = emit_leptos(&ast);
    });
    bench("leptos parse+emit (keys lowering)", ITERATIONS, || {
        let ast: ComponentAst = syn::parse2(keyed.clone()).expect("parses");
        let _ = emit_leptos(&ast);
    });
    bench("dioxus parse+emit (no keys)", ITERATIONS, || {
        let ast: ComponentAst = syn::parse2(plain.clone()).expect("parses");
        let _ = emit_dioxus(&ast);
    });
    bench("dioxus parse+emit (keys lowering)", ITERATIONS, || {
        let ast: ComponentAst = syn::parse2(keyed.clone()).expect("parses");
        let _ = emit_dioxus(&ast);
    });
}
