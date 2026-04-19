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
