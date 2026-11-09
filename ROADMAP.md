# quoin — Roadmap

This document outlines the planned development trajectory for the `quoin` project. It translates the specification suite (Vision, BRS, SRS, Architecture) into actionable phases with clear milestones and deliverables. This roadmap is a living document and will be updated as the project evolves.

---

## Guiding Principles

- **Quality First:** Every release must meet the quality standards defined in the SRS and pass the conformance suite.
- **Community Driven:** We prioritize features and adapters that serve the broader Rust UI ecosystem.
- **Semantic Versioning:** The core crate will adhere strictly to SemVer. Breaking changes require a major version bump.
- **Zero‑Cost Abstraction:** Performance is non‑negotiable. We will benchmark continuously.

---

## Phases Overview

| Phase | Focus | Target Outcomes |
|-------|-------|-----------------|
| **Phase 0: Foundation** | Core crate implementation, CI setup, initial documentation. | `quoin` core crate published (v0.1.x). |
| **Phase 1: Reference Adapters** | Implement and validate adapters for GPUI, Dioxus, Leptos. | Three official adapters passing conformance. |
| **Phase 2: Conformance & Community** | Release conformance test suite; onboard community adapters. | `quoin‑conformance` crate; adapters for Xilem, Floem. |
| **Phase 3: Ecosystem Growth** | Drive library adoption; gather feedback; iterate on core API. | At least two major libraries adopt `quoin`; stable 1.0 roadmap. |
| **Phase 4: Future Horizons** | `no_std` support, devtools, macros, additional runtimes. | Extensions based on community demand. |

---

## Phase 1.5: Depth & Completeness (Current)

**Goal:** Take the library to another dimension of completeness — expand the
API surface to cover the underlying frameworks' full capabilities, unify
cross-framework semantics, harden testing, and ship developer tooling.

### Milestones

#### M1.5.1: Core API Surface Expansion ✅
- [x] `derived()` / `.map()` / `zip()` computed signals (all adapters).
- [x] `Resource` async data primitive: `Idle → Loading → Ready | Failed`,
      race-free reloads, cooperative cancellation (`create_resource`).
- [x] `batch()` — collapse multiple signal writes into one update request
      (honoured by the GPUI and Xilem pull-based signals).
- [x] `spawn_local` via `LocalExecutor` for `!Send` futures (WASM platform
      loop; deterministic thread-local queue on native).
- [x] `CancellationToken` upgrades: `child_token()`, `guard()`,
      `into_cancelled()`, `run_until_cancelled()`.
- [x] `SignalExt::set_eq` / `replace` / `take`; `ExecutorExt::spawn_with_token`;
      `ReactiveContextExt::update_global` / `use_global_or_init`.

#### M1.5.2: Cross-Framework Semantics Unification ✅
- [x] **Shared globals everywhere**: `provide_global`/`use_global` now share
      one reactive cell on **all five adapters** (GPUI, Leptos, Dioxus, Xilem,
      Floem previously returned independent copies on four of them).
- [x] Identical `JoinHandle` lifecycle semantics re-verified by an expanded
      conformance suite (30+ checks per adapter, incl. locals and tokens).

#### M1.5.3: Performance ✅
- [x] Shared growable worker pool (`quoin_core::pool`) replaces one-OS-thread-
      per-task spawning in the Leptos/Dioxus/Floem executors (~20x cheaper
      spawn for short-lived tasks).
- [x] `set_eq` skips redundant writes; batch suppresses redundant notifies.
- [x] `quoin-core` micro-benchmark suite (`just bench-core`).

#### M1.5.4: Testing Infrastructure ✅
- [x] `quoin-testkit`: `TestContext` (update counting, shared globals),
      `ImmediateExecutor`, `ThreadedExecutor`, `TokioExecutor`,
      `LocalExecutor` support, `UpdateRecorder`.
- [x] `quoin-devtools`: `Instrumented` context wrapper, `EventLog`
      (`summary()`, `hot_signals()`, `dump()`), value snapshots, optional
      `tracing` integration, `QUOIN_TRACE` env gate.
- [x] Headless e2e examples: `todo-core` (full user sessions, render-churn
      assertions), `fetch-core` (resource lifecycle, reload cancellation),
      `devtools-demo` (trace session).

---

## Phase 1.6: Portable Reactivity (Current)

**Goal:** Complete the reactive triangle in `quoin-core` — push-based
reactions alongside the pull-based derived layer — and land the remaining
half of ADR-002, so framework-agnostic logic libraries can express "when
this changes, do that" without a framework effect system.

### Milestones

#### M1.6.1: Core Reactive Effects ✅
- [x] `quoin-core::effect` — dependency-tracked effects with automatic,
      per-run dependency sets (`create_effect`, `EffectHandle`,
      `EffectScope`, `watch`, `TrackedSignal`, `DepHandle`).
- [x] Batch integration: mutations inside `batch::with` coalesce into one
      effect re-run at batch end (the effect-side twin of the single
      `request_update`).
- [x] Zero-cost gates: one relaxed atomic load per read when idle; the
      dependency-cell lock is skipped entirely while no undisposed effect
      exists (verified by `just bench-core`, see `docs/BENCHMARKS.md`).
- [x] Detached-by-default handle semantics (runtime-owned registry,
      explicit `dispose()`), FIFO propagation, deferred self-writes,
      panic-propagation contract.
- [x] Tracked cells: `StaticSignal`, `NotifierSignal` (the GPUI/Xilem
      backing type), and the testkit `TestSignal` notify the core graph;
      globals share one dependency cell.

#### M1.6.2: Type-Erased Signals (ADR-002 second half) ✅
- [x] `AnySignal<T>` — opt-in dynamic dispatch for storing heterogeneous
      signal handles; `new_tracked` forwards the dependency cell; measured
      read overhead ≈ 0 ns (`docs/BENCHMARKS.md`).

#### M1.6.3: Conformance & Testkit ✅
- [x] `quoin-conformance::effects` — 11 portable checks plus the
      `define_effect_tests!` macro, executed against the testkit
      `TestContext` in CI.
- [x] `quoin-testkit` — `TestSignal` implements `TrackedSignal`; effect
      re-runs do not inflate the update counter; global handles share one
      dependency cell.

**Follow-ups (next milestone candidates):** effect conformance for the
fine-grained adapters once their signals bridge into the core graph;
a `effect!` macro emitter for Xilem backed by the core runtime (replacing the
`unsupported` row in `docs/BACKEND-PARITY.md`).

---

## Phase 1.7: Render-Layer Effect Parity (Current)

**Goal:** Make `effect!` first-class on every backend the `quoin_render!`
macro family targets, by wiring the Xilem emitter to the core reactive
runtime that Phase 1.6 landed — the last `unsupported` row in the
non-element contracts of `docs/BACKEND-PARITY.md`.

### Milestones

#### M1.7.1: The Xilem `effect!` emitter ✅
- [x] `quoin-xilem::xilem_effect` — `CleanupSlot` (the `Send + Sync`
      counterpart of `quoin::floem_effect::CleanupSlot`: cleanup runs before
      every re-run and a final time when the disposed effect node drops),
      the component effect-scope registry (thread-local scope stack), and
      the `effect!` entry point that routes into the active
      `quoin_core::EffectScope` or detaches outside a component mount.
- [x] `component!` on Xilem generates a `__quoin_effects` scope field;
      `on_mount` runs inside it, so `effect!` invocations register into the
      component's scope and are disposed when the component drops.
- [x] `effect!` in the render body is a named compile error (a rebuild
      would re-create the effect every time) — token-level detection with no
      false positives on locals named `effect` or comparisons.
- [x] Facade re-export `quoin::xilem_effect` (mirroring `floem_effect`).

#### M1.7.2: Conformance ✅
- [x] `define_effect_tests!` runs against the real `XilemContext` (11
      portable checks): the adapter's signals bridge into the core effect
      graph, so the shared contract holds without adapter code.
- [x] `define_effect_tests!` no longer imports `TestContextProvider`
      (fully qualified), so adapters can run both suites in one file.
- [x] Expansion tests pin the emitter shapes: core-runtime lowering,
      armed-cleanup slot, component-scope mount, render-body diagnostic.

#### M1.7.3: Benchmarks ✅
- [x] `quoin-xilem/examples/bench-effects.rs` (`just bench-xilem`): signal
      baseline, both effect-creation paths, the observed-write cycle,
      batch coalescing, the armed-cleanup shape, and `CleanupSlot::run`
      (see `docs/BENCHMARKS.md`).

**Follow-ups (next milestone candidates):** effect conformance for the
fine-grained adapters (Leptos, Dioxus, Floem) once *their* signals bridge
into the core graph — their `effect!` lowerings already target the
frameworks' own reactive runtimes, so this is a hardening candidate, not a
parity gap.

---

## Phase 1.8: Effect-Runtime Robustness (Current)

**Goal:** Harden the reactive kernel that Phase 1.6/1.7 made load-bearing:
a panicking effect body must leave the runtime exactly as it found it, a
failed creation must leave no trace, disposal must not degrade as detached
effects accumulate, and the ADR-002 erasure story must honour its documented
effect-graph contract.

### Milestones

#### M1.8.1: Unwind-safe effect runtime ✅
- [x] Tracking frames are RAII (`FrameGuard`): a panicking body pops its own
      frame on the way out — the thread's `TRACKING` gate is restored and no
      stale frame accumulates (before: every later tracked read on the
      thread paid full frame bookkeeping and leaked a `DepHandle` clone into
      the dead frame, forever).
- [x] `create_effect` rolls back on an initial-run panic
      (`RegistrationGuard`): no live effect, no registry entry, no disabled
      zero-cost gate (`LIVE_EFFECTS` stays balanced).
- [x] Panic contract documented in the module docs and pinned by tests:
      the effect keeps its previous dependency set until a successful run
      rebuilds it; `dispose()` remains the escape hatch.

#### M1.8.2: O(1) effect disposal ✅
- [x] The detached-effect registry is slot-indexed (`swap_remove` with slot
      fix-up under the registry lock); `dispose` no longer scans it —
      measured flat at ~68 ns with 100 or 1,000 live effects (was 147 ns at
      1,000 and linear; `docs/BENCHMARKS.md`).
- [x] Registry invariant (every entry's slot == its index) pinned by tests
      across interior/head/tail removal and re-creation.

#### M1.8.3: `TrackedSignal` for `AnySignal` ✅
- [x] The documented ADR-002 contract is now real: `AnySignal` implements
      `TrackedSignal`, forwarding the inner dependency cell when built with
      `new_tracked`; `watch()` over an erased handle re-runs on inner
      mutation.
- [x] Plain erasures (`AnySignal::new`) fail loudly with named guidance
      instead of silently never re-running.

#### M1.8.4: Prelude completion ✅
- [x] The prelude now really unlocks "the full framework-agnostic API in one
      import": effect runtime (`create_effect`, `watch`, `EffectHandle`,
      `EffectScope`, `DepHandle`, `TrackedSignal`), erasure
      (`NotifierSignal`), computed layer (`zip`/`Zipped`), batching
      (`BatchGuard`), and the task API (`spawn_abortable`,
      `spawn_local_task`, their handles, `run_local_tasks` /
      `pending_local_tasks`) — pinned by a compile-gate integration test.

**Follow-ups (next milestone candidates):** effect conformance for the
fine-grained adapters (bridge Leptos/Dioxus/Floem signals into the core
graph); `virtual_list` true virtualization on the web backends.

---

## Phase 1.9: Web-Emitter Contract Enforcement (Current)

**Goal:** Close the last silent-drop seam in the emitter surface: the web
reference backends (Leptos, Dioxus) parsed `keys { .. }` and silently
ignored it, violating the project's own contract vocabulary ("unsupported —
emits a named `compile_error!` at the element site, never a silent drop"),
and clear the remaining emitter-surface debt flagged by Phase 1.8.

### Milestones

#### M1.9.1: `keys { .. }` diagnostics on the web backends ✅
- [x] `component!` on Leptos and Dioxus emits the named `compile_error!`
      for `keys { .. }` (mirroring the Xilem contract): "no global key
      routing in v1; attach key events on the specific widget instead
      (see docs/BACKEND-PARITY.md)" — a real lowering (a `quoin::floem_keys`
      counterpart for the web) remains future work, and the diagnostic
      makes that a contract, not an accident.
- [x] Expansion gates `quoin-macros-core/tests/leptos_expansion.rs` and
      `dioxus_expansion.rs` (`[[test]] required-features`, mirroring the
      floem/xilem gates): the diagnostic fires, keys-free components stay
      clean, and the component lowerings (state init, hooks, Element
      fallback) are pinned.
- [x] `docs/BACKEND-PARITY.md` documents the shared web contract.

#### M1.9.2: Emitter-surface debt ✅
- [x] The four orphaned `transpile::*_codegen` modules
      (`dropdown_codegen`, `table_codegen`, `rich_text_codegen`,
      `virtual_list_codegen`) are deleted — superseded duplicates of the
      per-backend `emit::render_*` lowerings, referenced nowhere (ROADMAP
      Phase 1.8 follow-up "wire or delete": deleted).
- [x] `xilem_expansion.rs` no longer routes its `effect!` checks through
      the single-backend `Emitter` dispatch: the core-runtime lowering is a
      standalone `render_xilem::emit_effect`, so the expansion gate
      compiles under combined `floem,xilem` features (previously E0599 —
      CI only ran the parity-matrix target, never the whole combined test
      suite).

#### M1.9.3: Expansion-cost benchmarks ✅
- [x] `just bench-web-expansion`
      (`quoin-macros-core/examples/bench-web-expansion.rs`): parse + emit
      through both web emitters, keys-free vs keys-diagnostic components —
      the contract branch is unmeasurable at expansion time
      (`docs/BENCHMARKS.md`).

---

## Phase 1.10: Fine-Grained Effect Conformance (Current)

**Goal:** Close the last effect-parity seam the previous phases named as a
follow-up: the portable effect runtime (`quoin_core::create_effect`,
`watch`) could only observe the pull-based adapters (GPUI, Xilem), because
only their signal types carried a core dependency cell. The fine-grained
adapters' `effect!` lowerings already targeted the frameworks' own reactive
runtimes — this phase bridges their *signal types* into the core graph, so
framework-agnostic logic libraries get push-based reactions over
Leptos/Dioxus/Floem state too.

### Milestones

#### M1.10.1: The signal bridge ✅
- [x] `LeptosSignal`, `DioxusSignal`, and `FloemSignal` implement
      `TrackedSignal`: reads track a core dependency cell, writes notify it.
      The idle path stays zero-cost (one thread-local flag check per read);
      the frameworks' own reactivity is untouched.
- [x] `DioxusSignal` adopts the Leptos adapter's threading model: the
      underlying Dioxus signal sits in a `SendWrapper`, making the handle
      `Send` (core effect closures re-run on the notifying thread) while
      cross-thread dereference panics. Dioxus offers no public constructor
      for sync-storage signals, so the established adapter idiom replaces
      the previous compile-time `!Send`.
- [x] `provide_global`/`use_global` keep their shared-cell contract at the
      dependency level on Leptos and Dioxus: the shared dep cell rides in a
      second framework context, so a core effect observing one consumer's
      handle re-runs on writes through any other consumer's handle.

#### M1.10.2: Conformance ✅
- [x] `define_effect_tests!` (the 11 portable effect checks) now runs
      against the real `LeptosContext`, `DioxusContext`, and `FloemContext`
      — 33 new checks across the three adapters, alongside the Xilem suite
      from Phase 1.7. Every backend the `effect!` macro family targets is
      now covered by the shared effect contract.

#### M1.10.3: Benchmarks ✅
- [x] `just bench-leptos` / `bench-dioxus` / `bench-floem`
      (`examples/bench-effects.rs` in each adapter): signal baseline,
      bridged effect creation, the observed-write cycle, batch coalescing
      (see `docs/BENCHMARKS.md` — 227–418 ns observed cycles, idle reads
      unchanged).

**Follow-ups (next milestone candidates):** a real `keys` lowering for the
web backends (a `quoin::floem_keys` counterpart; the diagnostic contract
from Phase 1.9 stays in force until then); `virtual_list` true
virtualization on the web backends.

---

## Phase 1.11: Web Keys Lowering (Current)

**Goal:** Close the `keys { .. }` seam Phase 1.9 named as its follow-up:
the web backends (Leptos, Dioxus) carried the Phase 1.9 diagnostic because
no routing existed — while GPUI lowers `keys` through its action/bindings
system and Floem through `quoin::floem_keys`, and the Phase 1.10 signal
bridge had already made every handler-facing signal a first-class core-graph
citizen. What was missing was the last mile: one listener, one matcher, one
parse — all of which now exist behind the same contract vocabulary the
project enforces everywhere else.

### Milestones

#### M1.11.1: The shared keystroke runtime (`quoin::keys`) ✅
- [x] `quoin_core::keys` — the framework-free web keystroke vocabulary:
      `KeyEvent` (the W3C `code`/`key` snapshot plus the exact modifier
      mask), `keystroke_matches` (exact-modifier, case-insensitive on
      character keys — the same semantics as the Floem matcher), and
      `attach_window_keydown` → `KeydownHandle::remove`: one window-level
      `keydown` listener per component on `wasm32`, a documented no-op on
      native targets (SSR/desktop have no DOM window — the same platform
      boundary as the adapters' `clipboard_write_text`).
- [x] The facade re-exports the module explicitly (`pub use quoin_core::keys`)
      so the Floem adapter glob's own `keys` module cannot shadow it — the
      same explicit-twin pattern as `quoin::effect`.

#### M1.11.2: The web lowerings ✅
- [x] One shared macro-time parser (`emit::keystroke`) now feeds every
      backend that lowers `keys`: the GPUI grammar is parsed once into the
      exact modifier flags, the W3C code name, and the DOM key value; the
      Floem emitter maps the same parse onto native `NamedKey`/`Character`
      values (its expansion tokens unchanged — `floem_keys_expansion` and
      the parity matrix stay byte-identical), and the handler-arity probe
      moved to `emit::common` so no backend can drift.
- [x] `component!` on Leptos and Dioxus emits the real listener block:
      handler state shadow-cloned into the listener closure, one
      `if/else`-chained matcher arm per keystroke, cleanup wired through
      `on_cleanup` (Leptos) / `use_drop` (Dioxus). Malformed keystrokes
      still fail with the named `compile_error!` at the element site.
- [x] `docs/BACKEND-PARITY.md` documents the web contract (and keeps Xilem
      as the deliberate `unsupported` row).

#### M1.11.3: Gates & benchmarks ✅
- [x] Expansion gates `leptos_expansion` / `dioxus_expansion` pin the
      lowering: one listener per component, exact-modifier arms, the shared
      `quoin::ev::Key` payload, the cleanup hook, the named malformed
      diagnostic — and the keys-free expansion stays clean.
- [x] Trybuild compile gates `component_keys_leptos_pass` /
      `component_keys_dioxus_pass` compile the emitted code against the
      real framework stacks (native builds exercise the no-op attach).
- [x] `quoin-core` unit tests pin the matcher semantics; `just bench-core`
      gains the `keys.*` rows — 1–2 ns/op per keystroke, the same cost
      class as a hand-written `keydown` handler (`docs/BENCHMARKS.md`).

**Follow-ups (next milestone candidates):** dioxus-desktop webview key
routing (the desktop webview has a real window; the adapter's dependency
graph does not reach it yet); `virtual_list` true virtualization on the web
backends.

---

## Phase 0: Foundation (Current)

**Goal:** Implement the `quoin` core crate with the traits defined in the architecture specification, establish CI/CD, and publish initial development versions.

### Milestones

#### M0.1: Core Traits Implementation
- [ ] Define `ReactiveContext`, `Signal`, and `Executor` traits in `quoin` core.
- [ ] Implement `CancellationToken` utility for cooperative cancellation.
- [ ] Add compile‑time mutual exclusion for adapter feature flags.
- [ ] Write comprehensive doc comments with examples.

#### M0.2: Testing Infrastructure
- [ ] Set up GitHub Actions CI for Linux, macOS, Windows, and WASM targets.
- [ ] Add unit tests for `CancellationToken` and feature flag validation.
- [ ] Integrate `cargo audit`, `cargo deny`, and `rustfmt`/`clippy` checks.

#### M0.3: Initial Documentation
- [ ] Publish the specification suite (Vision, BRS, SRS, Architecture, Test Plan) in the repository.
- [ ] Create a `CONTRIBUTING.md` guide for adapter authors.
- [ ] Set up a simple GitHub Pages site with API docs (via `cargo doc`).

#### M0.4: First Release
- [ ] Publish `quoin` v0.1.0 to crates.io.
- [ ] Announce the project on Rust forums, Reddit, and social media.

---

## Phase 1: Reference Adapters

**Goal:** Implement and validate the core abstraction by creating production‑ready adapters for the three most widely used reactive frameworks: GPUI, Dioxus, and Leptos.

### Milestones

#### M1.1: GPUI Adapter (`quoin‑gpui`)
- [ ] Implement `ReactiveContext` for GPUI's `Context` and `Entity` system.
- [ ] Map `create_signal` to GPUI's `cx.new()` or similar patterns.
- [ ] Map `Executor::spawn` to `cx.spawn()`.
- [ ] Ensure `request_update` calls `cx.notify()`.

#### M1.2: Dioxus Adapter (`quoin‑dioxus`)
- [ ] Implement `ReactiveContext` for Dioxus's `Scope` and `Signal`.
- [ ] Map `create_signal` to `use_signal`.
- [ ] Map `Executor::spawn` to `spawn` (or `wasm_bindgen_futures::spawn_local`).
- [ ] Ensure `request_update` is a no‑op (Dioxus is automatically reactive).

#### M1.3: Leptos Adapter (`quoin‑leptos`)
- [ ] Implement `ReactiveContext` for Leptos's `RwSignal` and `create_effect`.
- [ ] Map `create_signal` to `create_rw_signal`.
- [ ] Map `Executor::spawn` to `spawn_local`.
- [ ] Ensure `request_update` is a no‑op.

#### M1.4: Adapter Validation
- [ ] Develop the conformance test suite (see Phase 2) and run it against all three adapters.
- [ ] Create a simple example app that uses a dummy `quoin`‑based library with each adapter to validate end‑to‑end integration.
- [ ] Publish `quoin‑gpui`, `quoin‑dioxus`, and `quoin‑leptos` v0.1.0 to crates.io.

---

## Phase 2: Conformance Suite & Community Adapters

**Goal:** Release a reusable conformance test crate that enables community members to confidently build and validate new adapters. Foster a community of adapter maintainers.

### Milestones

#### M2.1: Conformance Crate (`quoin‑conformance`)
- [ ] Create a separate crate using `tested‑trait` (or custom macro) to define the conformance tests.
- [ ] Implement tests for: signal creation/reading, mutable updates, executor spawning, cancellation, and `request_update` behavior.
- [ ] Publish `quoin‑conformance` v0.1.0 with documentation on how adapter authors can use it.

#### M2.2: Community Onboarding
- [ ] Create an `quoin‑adapters` GitHub repository (or a section in the main repo) to index community adapters.
- [ ] Define a clear process for submitting an adapter: pass conformance suite, open PR to add to index.
- [ ] Add badges (shields.io) to indicate conformance status.

#### M2.3: Xilem & Floem Adapters (Community Focus)
- [ ] Encourage community contributions for `quoin‑xilem` and `quoin‑floem`.
- [ ] Provide mentorship and code review for initial community adapters.
- [ ] (Stretch) Core team implements one of these as a reference for the community.

---

## Phase 3: Ecosystem Growth

**Goal:** Achieve the business outcomes defined in the Vision and BRS: adoption by major libraries, framework coverage, and ecosystem influence.

### Milestones

#### M3.1: Library Adoption
- [ ] Work with maintainers of `rs‑query` and `navi` to integrate `quoin` (as defined in the operational scenarios).
- [ ] Publish case studies or blog posts detailing the integration experience and performance benchmarks.
- [ ] Target: ≥3 distinct libraries depending on `quoin`.

#### M3.2: Stable API & 1.0 Roadmap
- [ ] Gather feedback from early adopters on the core traits.
- [ ] Identify any necessary breaking changes and plan a path to 1.0.
- [ ] Finalize `Signal` vs `MutableSignal` design.
- [ ] Publish a 1.0 release candidate and solicit final review.

#### M3.3: Ecosystem Advocacy
- [ ] Present `quoin` at Rust conferences and meetups (RustConf, RustNL, EuroRust).
- [ ] Write articles for This Week in Rust and Rust blog.
- [ ] Engage with framework maintainers to encourage official adapter support.

---

## Phase 4: Future Horizons

**Goal:** Expand `quoin`'s capabilities based on deferred requirements and community demand. These items are **not** on the critical path to 1.0.

| Feature | Description | Priority |
|---------|-------------|----------|
| **`no_std` Support** | Enable `quoin` in embedded and bare‑metal environments. | Medium |
| **Devtools Message Bus** | Provide optional instrumentation for signal updates and task lifecycle, enabling a `quoin‑devtools` panel. | Medium |
| **Proc‑Macro Ergonomics** | Create `#[derive(Signal)]` or similar macros to reduce boilerplate for adapter authors. | Low |
| **Additional Executor Backends** | Support for `smol`, `async‑std`, etc. | Low |
| **FFI Bindings** | Expose `quoin` concepts to C/C++ for use in other language UI frameworks. | Very Low |

---

## How to Contribute

We welcome contributions at any phase! Check the [CONTRIBUTING.md](CONTRIBUTING.md) guide and look for issues labeled `good first issue` or `help wanted`. The best way to get involved in the current phase is to help implement or test the core traits and reference adapters.
