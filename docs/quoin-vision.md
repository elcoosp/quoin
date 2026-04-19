# Product Vision & Strategic Alignment — quoin

| Field | Value |
|-------|-------|
| Project | quoin |
| Document | Product Vision & Strategic Alignment |
| Version | 0.1 (Draft) |
| Date | 2026-04-19 |
| Author | User, assisted by AI |
| Status | Draft — Pending Review |

---

## 1. Vision Statement

> *"One reactive core, many frameworks. `quoin` makes it trivial for Rust developers to build framework‑agnostic libraries and applications by abstracting away UI‑specific reactivity behind feature flags."*

## 2. Elevator Pitch

> *"For Rust library authors who are tired of maintaining separate reactive integrations for each UI framework, `quoin` is a foundational reactive abstraction crate that provides a single, unified API with framework‑specific adapters. Unlike ad‑hoc framework bindings, `quoin` enables true write‑once‑run‑anywhere via compile‑time feature flags."*

## 3. Problem Statement & Business Context

The Rust UI ecosystem is maturing rapidly, but its reactive foundation remains balkanized. Every major framework—GPUI, Dioxus, Leptos, Xilem, Floem—implements its own primitives for signals, effects, context, and execution. Without a shared abstraction, each framework becomes an island. This forces library authors who want to support multiple frameworks to either pick a single ecosystem and exclude others, or maintain bespoke integrations for each. The result is duplicated effort, vendor lock‑in for application developers, and friction that slows the entire ecosystem's growth. `quoin` provides a pragmatic, compile‑time‑oriented solution: a single, shared reactive abstraction layer that libraries and applications can use to achieve true framework‑agnosticism.

## 4. Target Users & Customers

### Primary Audience
`quoin` serves both **library authors** (who will directly consume `quoin`'s traits to build framework‑agnostic crates) and **application developers** (who will indirectly benefit through libraries that use `quoin`, gaining freedom to choose or switch UI frameworks).

### Explicitly Out of Scope (Non‑Target Users)
- **Embedded / `no_std` environments** — Initial releases target standard library environments; `no_std` support is deferred.
- **Non‑Rust language consumers (FFI)** — `quoin` is a pure Rust abstraction layer; C/C++/other language bindings are not planned.
- **Framework maintainers** — `quoin` does not prescribe how frameworks should implement reactivity internally; it provides an abstraction for *consumers* of framework reactivity (libraries and applications).

## 5. User Needs & Value Proposition

### Core User Needs
`quoin` addresses the following needs:

1. **Write once, support many frameworks** — A single reactive API surface that works consistently across GPUI, Dioxus, Leptos, Xilem, Floem, and future frameworks, activated via compile‑time feature flags.

2. **Minimal boilerplate for adapter authors** — Implementing a new framework adapter should be trivial; the core crate encapsulates the majority of the complexity, requiring only thin integration layers.

3. **Compile‑time safety with zero runtime overhead** — Feature flags ensure only the selected framework's code is compiled; abstractions are zero‑cost and optimized away where possible.

4. **Reactive primitives that feel native** — Signals, effects, and context propagation behave idiomatically within each target framework while sharing a common trait‑based interface.

5. **Unified async runtime abstraction** — A consistent way to spawn and manage asynchronous tasks across GPUI's foreground executor, Tokio, and `wasm‑bindgen` futures.

6. **Ecosystem interoperability** — Libraries built on `quoin` (e.g., `rs‑query` for state management, `navi` for routing) compose seamlessly without either being tied to a specific framework.

### Value Proposition & Differentiator

> *Unlike ad‑hoc framework bindings that require bespoke code for each integration, `quoin` provides a single, unified trait‑based abstraction that eliminates the N×M integration problem via feature‑flagged write‑once‑run‑anywhere. Moreover, `quoin` is purpose‑built as a foundation layer: multiple libraries (`rs‑query`, `navi`, and others) can build upon it, creating a network effect of framework‑agnostic tools that benefit the entire Rust UI ecosystem.*

## 6. Desired Outcomes & Success Metrics

`quoin` will be considered successful when it achieves the following within 18–24 months of its first stable release.

### Business Outcomes

| ID | Outcome | Key Results |
|----|---------|-------------|
| **OUT‑1** | **Library Adoption** — At least two major Rust libraries adopt `quoin` as their reactive foundation. | **KR‑1.1:** ≥ 3 distinct crates depend on `quoin` (excluding adapters). <br/> **KR‑1.2:** ≥ 50,000 combined downloads on crates.io. |
| **OUT‑2** | **Framework Coverage** — Stable, production‑ready adapters exist for at least four major Rust UI frameworks. | **KR‑2.1:** ≥ 4 adapter crates with ≥ 1.0.0 release. <br/> **KR‑2.2:** 100% pass rate on shared conformance test suite. |
| **OUT‑3** | **Ecosystem Influence** — `quoin`'s core traits are recognized as a de facto standard abstraction layer. | **KR‑3.1:** ≥ 5 external references in RFCs, blog posts, or conference talks. <br/> **KR‑3.2:** ≥ 500 GitHub stars and ≥ 10 unique contributors. |

## 7. Strategic Constraints

| Constraint | Description |
|------------|-------------|
| **Platform support** | Must compile on Linux, macOS, Windows, and WebAssembly (WASM). Mobile (iOS/Android) desired but not required for initial release. |
| **Rust version compatibility** | Must support current stable Rust and maintain compatibility with at least the two most recent stable releases. |
| **License** | Dual‑licensed under MIT and Apache‑2.0, with MIT as the primary choice. |
| **Zero‑cost abstractions** | Core traits and feature‑flagged adapters must compile to zero or near‑zero runtime overhead. No dynamic dispatch unless explicitly opted into. |
| **Async runtime neutrality** | Must not hard‑code a dependency on a specific async runtime. Works with GPUI's foreground executor, Tokio, `wasm‑bindgen‑futures`, and others via the `Executor` trait. |
| **No mandatory proc‑macros** | Core functionality must be usable without procedural macros. Optional macros may be added for ergonomics. |
| **Minimal dependencies** | Core crate aims for zero or very few dependencies beyond `std` and foundational crates like `futures‑core`. |

## 8. Goals and Non‑Goals

### Goals

1. Provide a unified `ReactiveContext` trait that abstracts framework‑specific reactivity (signals, effects, context).
2. Provide a unified `Executor` trait for spawning async tasks across different runtimes.
3. Enable library authors to write reactive logic once and support multiple frameworks via compile‑time feature flags.
4. Offer reference implementations (adapters) for GPUI, Dioxus, Leptos, Xilem, and Floem to prove the abstraction and serve as examples.
5. Maintain a stable core API with semantic versioning to allow libraries to depend on `quoin` confidently.
6. Foster an ecosystem where multiple framework‑agnostic libraries can interoperate.

### Non‑Goals (Explicitly Excluded)

1. **Not a widget abstraction layer** — `quoin` does not attempt to unify UI components or rendering across frameworks.
2. **Not a framework itself** — `quoin` is not a new UI framework; it is a foundation for libraries that need framework‑agnostic reactivity.
3. **Not a replacement for framework‑native reactivity** — `quoin` does not dictate how frameworks should implement their internal reactivity; it only provides a consumer‑side abstraction.
4. **No runtime framework switching** — Feature flags select a framework at compile time; there is no dynamic dispatch or runtime selection of framework backends.
5. **No initial `no_std` support** — Standard library required; `no_std` is deferred.
6. **No initial FFI bindings** — Pure Rust only; foreign language bindings are out of scope.
7. **No built‑in devtools** — Devtools message bus and panels are deferred to a future crate or version.

## 9. Operational Concept & High‑Level Scenarios

### Scenario A: Library Author Building a New Framework‑Agnostic Crate
The author defines their core reactive logic using `quoin` traits. They add feature flags for each framework they want to support. Downstream users select a feature flag, and the library works natively with that framework's reactivity system.

### Scenario B: Application Developer Using Framework‑Agnostic Libraries
The developer chooses GPUI for their app. They add `rs‑query` with the `gpui` feature flag. `rs‑query` returns GPUI `Entity` objects that work seamlessly with existing GPUI components. Later, they consider switching to Dioxus; they change the feature flag and rebuild, and the same `rs‑query` API now returns Dioxus `Signal` objects.

### Scenario C: Framework Maintainer Evaluating `quoin`
The maintainer reviews the `ReactiveContext` trait and realizes they can implement it for their framework in under 200 lines of code, gaining immediate compatibility with `rs‑query`, `navi`, and any other `quoin`‑based library.

### Scenario D: Community Contributor Adding a New Framework Adapter
A contributor notices that `quoin` doesn't yet support a new framework called `Vizia`. They implement the `ReactiveContext` trait, submit a PR, and within days, all `quoin`‑compatible libraries now work with `Vizia`.

## 10. Stakeholders, Sponsorship, and Governance

| Role | Description |
|------|-------------|
| **Primary Owner / Lead Maintainer** | User (project creator) — responsible for overall direction, API stability, and final decision authority. |
| **Governance Model** | Benevolent Dictator for Life (BDFL) — The lead maintainer has final decision authority, with input from the community and contributors. |
| **Change Approval** | Major changes to vision, scope, or non‑goals require BDFL sign‑off. Minor changes and adapter updates follow standard PR review. |

## 11. Risks, Assumptions, and Open Questions

### Critical Risks

| Risk | Mitigation |
|------|------------|
| **Breaking changes in target frameworks** — A framework changes its reactivity model in a way `quoin` cannot accommodate. | Maintain close relationships with framework maintainers; design traits to be minimal and forward‑compatible; version adapters independently of core. |
| **Performance overhead** — Trait‑based abstraction introduces measurable runtime cost. | Rigorous benchmarking; zero‑cost abstraction design; static dispatch by default; community review of generated assembly. |
| **Async runtime diversity** — Supporting multiple executors proves leaky or insufficient. | Start with a minimal `Executor` trait covering only spawn; use `CancellationToken` utility for cooperative cancellation rather than abstracting executor‑specific semantics. |

### Assumptions

- The major Rust UI frameworks will continue to expose similar reactive primitives (signals, effects, context) that can be unified under a common trait interface.
- Library authors are willing to adopt a new abstraction layer if it demonstrably reduces maintenance burden and expands their user base.
- Compile‑time feature flags are an acceptable mechanism for framework selection; runtime switching is not required.

### Open Questions & Recommended Direction

| Question | Recommended Direction | Evidence Basis |
|----------|----------------------|----------------|
| **Minimal viable set of traits** | `Executor`, `Signal<T>`, `ReactiveContext` | Existing executor abstraction crates (`agnostic_async_executor`, `some_executor`); Leptos `Signal` wrapper pattern. |
| **How should signals be modeled?** | Trait‑based core (`Signal<T>`) with optional wrapper struct for dynamic dispatch. | Leptos wrapper + `ankurah_signals` trait pattern; supports both zero‑cost static dispatch and flexibility. |
| **Testing strategy for adapters** | Use `tested-trait` to define a conformance suite that every adapter must pass. | `conformer` and `tested-trait` crates provide patterns for testing multiple trait implementations. |
| **Async cancellation handling** | Provide a `CancellationToken` utility; do not attempt to abstract executor‑specific cancellation semantics in the core `Executor` trait. | `tokio_util::sync::CancellationToken` pattern; `some_executor` documentation on cancellation variance. |
| **Migration path for existing libraries** | Gradual adoption via feature‑flag parallel implementations; deprecate native integrations after sufficient adoption. | Multi‑backend pattern used by `tui‑realm` (supports `crossterm` and `termion` via feature flags). |
| **Devtools placement** | Defer to a separate `quoin-devtools` crate; core provides optional instrumentation behind a `devtools` feature flag. | Chrome DevTools Protocol architecture; Rust channel patterns for in‑process messaging. |
