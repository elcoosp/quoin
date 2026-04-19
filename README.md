# quoin

> *"One reactive core, many frameworks."*

**`quoin`** is the foundational reactive abstraction layer for the Rust UI ecosystem. It enables library authors to write reactive logic **once** and support multiple UI frameworks—GPUI, Dioxus, Leptos, Xilem, Floem, and beyond—with only a feature flag toggle.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](#)
[![Status: Specification](https://img.shields.io/badge/status-specification-lightgrey.svg)]()

---

## 📖 Table of Contents

- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [Architecture at a Glance](#-architecture-at-a-glance)
- [Specification Suite](#-specification-suite)
- [Project Status](#-project-status)
- [Getting Involved](#-getting-involved)
- [License](#-license)

---

## 🚨 The Problem

The Rust UI ecosystem is **balkanized**. Every major framework—GPUI, Dioxus, Leptos, Xilem, Floem—implements its own primitives for signals, effects, context, and async execution. For library authors, this means:

- **Duplicated effort:** Maintaining bespoke integrations for each framework.
- **Vendor lock‑in:** Application developers are trapped in a single ecosystem.
- **Fragmentation:** A rich library for one framework doesn't benefit others.

`quoin` fixes this.

---

## 💡 The Solution

`quoin` provides a **single, shared reactive abstraction** built on three core traits:

| Trait | Purpose |
|-------|---------|
| **`ReactiveContext`** | The framework's reactive runtime. Creates signals and provides an executor. |
| **`Signal<T>`** | A readable (and optionally writable) reactive value. |
| **`Executor`** | Spawns asynchronous tasks on the framework's native runtime. |

**Write once, run anywhere:**

```rust
// Your library's core logic uses `quoin` traits.
fn use_counter<C: ReactiveContext>(cx: &C) -> impl Signal<u32> {
    cx.create_signal(0)
}
```

**Downstream users select a framework with a feature flag:**

```toml
# Cargo.toml
[dependencies]
my-agnostic-lib = { version = "1.0", features = ["gpui"] }
```

At compile time, only the code for the chosen framework is included. **Zero runtime overhead.**

---

## 🏗️ Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                    Downstream Library                        │
│  (e.g., rs‑query, navi)                                     │
│  - Depends on `quoin`                                       │
│  - Selects one adapter via feature flag                      │
└─────────────────────────┬───────────────────────────────────┘
                          │
          ┌───────────────┴───────────────┐
          │                               │
┌─────────▼─────────┐             ┌───────▼───────┐
│    quoin (core)   │             │  quoin‑gpui   │
│  - ReactiveContext│◄────────────│  (adapter)    │
│  - Signal<T>      │  implements │               │
│  - Executor       │             └───────────────┘
└───────────────────┘
          ▲
          │ implements
┌─────────┴─────────┐     ┌───────────────┐     ┌───────────────┐
│  quoin‑dioxus     │     │  quoin‑leptos │     │  quoin‑xilem  │
└───────────────────┘     └───────────────┘     └───────────────┘
                              ▲
                              │ (community adapters welcome!)
```

All adapters are **equal**. Any crate that implements `ReactiveContext` and passes the conformance test suite is a first‑class `quoin` adapter.

---

## 📚 Specification Suite

The `quoin` project is guided by a complete, evidence‑backed specification suite. Each document is traceable to the next, forming a rigorous foundation for implementation.

| Document | Description | Key Artifacts |
|----------|-------------|---------------|
| **[Vision & Strategic Alignment](docs/vision.md)** | The "why" – problem statement, target users, success metrics, non‑goals. | Vision statement, elevator pitch, OKRs, strategic constraints. |
| **[Business & Stakeholder Requirements (BRS)](docs/brs.md)** | The "what the business needs" – stakeholder goals, business rules, operational concept. | Business goals with fit criteria, user classes, glossary, conceptual domain model. |
| **[Software Requirements Specification (SRS)](docs/srs.md)** | The "what the system does" – functional and non‑functional requirements, verifiable and prioritized. | EARS‑style requirements, NFRs with measurable targets, external interface contracts. |
| **[Architecture & Design Specification](docs/architecture.md)** | The "how it works" – structural design, key decisions (ADRs), API contracts. | C4 diagrams, ADRs (trait design, executor abstraction, conformance suite), API trait definitions. |
| **[Behavioral Spec & Test Verification](docs/test-verification.md)** | The "prove it" – BDD scenarios, conformance test suite, traceability matrix. | Gherkin scenarios, test strategy, Requirements Traceability Matrix (RTM). |

📁 **All documents are available in the [`docs/`](docs/) directory.**

---

## 🚦 Project Status

| Phase | Status |
|-------|--------|
| **Specification** | ✅ Complete — Vision, BRS, SRS, Architecture, Test Plan |
| **Core Implementation** | 🔜 Pending — `quoin` core crate |
| **Reference Adapters** | 🔜 Pending — GPUI, Dioxus, Leptos |
| **Conformance Suite** | 🔜 Pending — `quoin‑conformance` |
| **Community Adapters** | ⏳ Future — Xilem, Floem, Vizia, etc. |

**Current Focus:** Validating the core trait design with a proof‑of‑concept implementation.

---

## 🤝 Getting Involved

`quoin` is a community‑driven project. We welcome contributions of all kinds!

### For Library Authors
- **Adopt `quoin`:** Make your crate framework‑agnostic. See the [Architecture Spec](docs/architecture.md) for API details.
- **Provide feedback:** Open an issue to discuss your use case or pain points.

### For Framework Enthusiasts
- **Write an adapter:** Implement `ReactiveContext` for your favorite UI framework. The [SRS](docs/srs.md) and [Test Spec](docs/test-verification.md) define the exact contract.
- **Get listed:** Once your adapter passes the conformance suite, submit a PR to have it listed in the official adapter index.

### For Everyone
- **Spread the word:** Star the repo, share the vision, and help us build the unified reactive foundation Rust deserves.
- **Read the specs:** Familiarize yourself with the project's goals and constraints.

### Governance
`quoin` operates under a **BDFL** model. The project lead has final decision authority, with input from the community. All crates are dual‑licensed under MIT and Apache‑2.0. See the [Vision](docs/vision.md) for full governance details.

---

## 📄 License

All `quoin` crates are dual‑licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

---

*Built with ❤️ for the Rust UI ecosystem.*
