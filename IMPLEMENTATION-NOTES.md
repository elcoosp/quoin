# Implementation notes — gpui-rsx migration Phases 0–4

This tree implements Phases 0–4 of the gpui-rsx migration plan
(`quoin-gpui-rsx-migration-report.md` §8 / `docs/gpui-rsx-classes.md`),
committed on top of upstream `main` (7763027, "docs: add the rendering-parity
milestone report"). The default engine is untouched — everything ships
behind the `gpui-rsx-classes` feature flag.

## What was implemented

| Phase | Deliverable | Files |
| --- | --- | --- |
| 0 | Parity harness as committed workspace crate: 34/34 theme-token byte-parity, vocabulary-resolvability gate, Tailwind-scale gate, variant table + 2 diagnostic probes | `interop-tests/` |
| 1 | Class-engine swap (strict, Tailwind scale) for classes quoin rejects; feature plumbing through `quoin-macros` and the `quoin` facade | `quoin-macros-core/src/transpile/tailwind_grsx.rs`, `Cargo.toml`s, `emit/render_gpui/mod.rs` |
| 2 | `rsx.toml` theme extension at expansion time (`Theme::load_from_dir` + `include_bytes!` rebuild tracking); token classes keep baked hexes via the quoin-first policy | `tailwind_grsx.rs` (`user_theme`) |
| 3 | Dynamic `class:` expressions resolve at render time — inline `rsx_runtime` module (gpui-rsx-core's generated matcher) hoisted once per `quoin_render!` expansion | `emit/render_gpui/mod.rs` (`emit_render_root`, `dynamic_runtime_module`), `transpile/tailwind.rs` (`finish`) |
| 4 | `rsx!` escape hatch: gpui-rsx's macros re-exported from the facade (`gpui` feature enables; standalone `rsx` feature) | `quoin/src/lib.rs`, `quoin/Cargo.toml` |
| — | Compile-only verification crate against the vendored GPUI stub | `quoin-grsx-compiletest/`, `third_party/gpui-stub*` |
| — | Committed minimal `muda` resolution stub (upstream keeps the full stub untracked; fresh clones could not resolve the workspace) | `third_party/muda-stub/` |

## Verification evidence (all run in this tree)

| Suite | Result |
| --- | --- |
| `cargo test -p quoin-interop-tests` (Phase-0 gates) | 11/11 green — 34/34 theme classes byte-identical, every quoin class resolvable by the engine (3 documented exceptions), Tailwind `p-4` → 16 px, variants |
| `cargo test -p quoin-macros-core --features gpui,gpui-rsx-classes` | 20/20 green — decision policy per class, hex/table goldens, variant routing, dynamic constructor, unknown-class errors with suggestions |
| `cargo test -p quoin-macros-core` / `-p quoin-style` | 4/4 and 21/21 green — default engine untouched |
| floem/xilem expansion + parity matrix | 8/9/5 green |
| `cargo check -p quoin-grsx-compiletest` + `cargo run` | compiles AND executes — full expansions (new vocabulary, variants, dynamic, control flow, handlers) type-check against the gpui-stub generated from the same gpui-pre@0.3.10 snapshot |
| `cargo check -p quoin --features gpui,gpui-rsx-classes` (real gpui-pre 0.3.10 + gpui-component 0.8.1) | green (2 m 44 s) |
| `cargo check -p quoin --features gpui` / `--features rsx` / `--features macros` | green |
| `cargo clippy` on every touched crate, both feature combos | clean (pedantic, `-D warnings` policy) |
| `cargo test -p quoin-macros-tests --features gpui` (trybuild UI tests) | **not runnable here** — trybuild's isolated target needs ~5 GB beyond this sandbox's disk; unaffected by the flag-off path (byte-parity is proven by the unit tests). Run in CI: `just test-quoin-macros-gpui` and `just test-quoin-macros-gpui-rsx-classes` |

## Findings recorded during implementation (all pinned by tests)

1. `gpui_rsx_core` emits **bare** `px/rems/relative/rgb/rgba/FontWeight` — the
   engine bridge qualifies them to `::gpui::…` (`qualify_gpui_values`) because
   quoin's macro output cannot assume a `use gpui::*` at the call site.
2. gpui-rsx-core **0.10.2** rejects negative utilities (`-mt-2`) in strict
   mode with its own guidance; upstream main added them later (#108). Not a
   regression — quoin rejected them too.
3. `hover:` keeps quoin's exact `.hover(|__s| __s …)` shape; the new variants
   ride the normal list as fully-wrapped refinements and set `stateful`, which
   now shares the `.id(..)` injection with scroll via
   `TranspiledStyles::needs_stateful()`.
4. `syn` version split (quoin on syn 3, gpui-rsx-core on syn 2) is bridged by
   a renamed optional dep (`syn2`), needed only for `generate_runtime_module`'s
   typed path argument.

## Not done (by design — plan Phase 5)

- The default engine is NOT flipped: classes quoin knows keep the current
  lowering byte-for-byte. The flip (engine-first everywhere, retire
  `decl_to_gpui_tokens`' GPUI leg) is the next release's work, gated on real
  apps running with the flag.
- Runtime rendering behavior is not verified in this sandbox (no display
  server); run an example app (`examples/counter-gpui`) with
  `quoin/gpui-rsx-classes` before flipping the default.
