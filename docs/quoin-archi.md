# Architecture & Design Specification — quoin

| Field | Value |
|-------|-------|
| Project | quoin |
| Document | Architecture & Design Specification |
| Version | 0.1 (Draft) |
| Date | 2026-04-19 |
| Author | User, assisted by AI |
| Status | Draft — Pending Review |

---

## Table of Contents

1. [Introduction](#1-introduction)
   - 1.1 Purpose
   - 1.2 Scope
   - 1.3 References
2. [Goals and Non‑Goals](#2-goals-and-non‑goals)
3. [Architecturally Significant Requirements (ASRs)](#3-architecturally-significant-requirements-asrs)
4. [System Overview and High‑Level Structure](#4-system-overview-and-high-level-structure)
5. [C4 Model Views](#5-c4-model-views)
   - 5.1 System Context (C1)
   - 5.2 Container Diagram (C2)
   - 5.3 Component Diagram (C3) — Core Crate Internals
6. [Architecture Decision Records (ADRs)](#6-architecture-decision-records-adrs)
   - ADR‑001: Trait‑Based Reactive Abstraction with Feature Flags
   - ADR‑002: Signal Trait Design – Static Dispatch vs. Dynamic Dispatch
   - ADR‑003: Executor Abstraction and Cancellation Strategy
   - ADR‑004: Conformance Test Suite Using `tested‑trait`
   - ADR‑005: Adapter Crate Organization
7. [API and Interface Contracts](#7-api-and-interface-contracts)
   - 7.1 `ReactiveContext` Trait
   - 7.2 `Signal<T>` Trait
   - 7.3 `Executor` Trait
8. [Cross‑Cutting Concerns](#8-cross‑cutting-concerns)
   - 8.1 Observability
   - 8.2 Error Handling
   - 8.3 Documentation Generation
9. [Deployment and Build Considerations](#9-deployment-and-build-considerations)
10. [Alternatives Considered](#10-alternatives-considered)
11. [Traceability: ASRs to Design Decisions](#11-traceability-asrs-to-design-decisions)
12. [Risks and Open Issues](#12-risks-and-open-issues)

---

## 1. Introduction

### 1.1 Purpose
This Architecture & Design Specification describes the structural and behavioral design of the `quoin` system—a foundational reactive abstraction layer for Rust UI frameworks. It documents the key architectural decisions, component structures, interface contracts, and cross‑cutting concerns. The intended audience includes core maintainers, framework adapter contributors, and downstream library authors who need to understand the design rationale.

### 1.2 Scope
This document covers:
- The `quoin` core crate (`quoin`).
- The reference framework adapter crates (`quoin‑gpui`, `quoin‑dioxus`, `quoin‑leptos`, `quoin‑xilem`, `quoin‑floem`).
- The conformance test suite (`quoin‑conformance`).
- Build and feature‑flag architecture.

It does **not** cover:
- Devtools, `no_std` support, or FFI bindings (deferred per Vision Non‑Goals).
- Internal implementation details of the target UI frameworks.

### 1.3 References
- `quoin` Product Vision & Strategic Alignment (v0.1)
- `quoin` Business & Stakeholder Requirements Specification (v0.1)
- `quoin` Software Requirements Specification (v0.1)
- ISO/IEC/IEEE 42010:2022 — Architecture description
- The C4 Model for Visualising Software Architecture

---

## 2. Goals and Non‑Goals

### 2.1 Architectural Goals
| ID | Goal | Rationale |
|----|------|-----------|
| **ARCH‑GOAL‑01** | **Zero‑Cost Abstraction** | Traits and generics must compile to code equivalent to hand‑written framework‑specific code. |
| **ARCH‑GOAL‑02** | **Minimal Core Dependencies** | Core crate depends only on `std` and `futures‑core` to ensure broad compatibility. |
| **ARCH‑GOAL‑03** | **Compile‑Time Adapter Selection** | Feature flags enforce that only one framework adapter is compiled into a downstream binary. |
| **ARCH‑GOAL‑04** | **Adapter Conformance Validation** | A shared test suite guarantees consistent behavior across all adapters. |
| **ARCH‑GOAL‑05** | **Low Adapter Implementation Complexity** | The `ReactiveContext` trait is minimal, enabling new adapters in under 200 LOC. |

### 2.2 Architectural Non‑Goals
| ID | Non‑Goal | Rationale |
|----|----------|-----------|
| **ARCH‑NON‑01** | **Runtime Framework Switching** | Framework selection occurs at compile time; dynamic dispatch is avoided. |
| **ARCH‑NON‑02** | **Widget or Component Abstraction** | `quoin` focuses solely on reactive state and async execution, not UI rendering. |
| **ARCH‑NON‑03** | **`no_std` Support (Initial)** | Initial releases target `std` environments; `no_std` is deferred. |
| **ARCH‑NON‑04** | **Built‑in Devtools** | Devtools are deferred to a separate crate. |

---

## 3. Architecturally Significant Requirements (ASRs)

These requirements from the SRS have a measurable effect on the architecture.

| ASR ID | SRS Requirement | Architectural Impact |
|--------|----------------|----------------------|
| **ASR‑001** | NFR‑PERF‑001: Zero‑Cost Abstraction | Drives use of static dispatch (`impl Trait`) and `#[inline]` annotations. |
| **ASR‑002** | REQ‑FUNC‑030‑033: Feature‑Flagged Adapter Selection | Mandates a feature‑flag architecture and mutually exclusive adapter features. |
| **ASR‑003** | NFR‑COMP‑003: Async Runtime Neutrality | Influences `Executor` trait to be minimal; cancellation is handled via `CancellationToken` utility. |
| **ASR‑004** | NFR‑MAIN‑002: Minimal Core Dependencies | Constrains `Cargo.toml` to `futures‑core` only. |
| **ASR‑005** | NFR‑REL‑001: Adapter Conformance Test Pass Rate | Requires a reusable conformance test suite crate. |
| **ASR‑006** | NFR‑UX‑001: Adapter Implementation ≤ 200 LOC | Guides the API surface of `ReactiveContext` to be as small as possible. |

---

## 4. System Overview and High‑Level Structure

`quoin` is organized as a **core crate** plus a set of **independent adapter crates**. Downstream libraries depend on the core and select exactly one adapter via Cargo feature flags.

**Key Architectural Principles:**
- **Trait‑based abstraction:** The core defines traits; adapters implement them.
- **Compile‑time binding:** Feature flags determine which adapter implementation is linked.
- **Zero‑cost generics:** All trait methods use static dispatch where possible.

**Crate Relationships:**
```
┌─────────────────────────────────────────────────────────────┐
│                    Downstream Library                        │
│  (e.g., rs‑query)                                           │
│  - depends on `quoin`                                       │
│  - selects one adapter via feature flag (e.g., `gpui`)      │
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
```

---

## 5. C4 Model Views

### 5.1 System Context (C1)

**Description:** `quoin` is a library consumed by downstream crates (Library Authors). It integrates with target UI frameworks (GPUI, Dioxus, Leptos, Xilem, Floem) through adapter crates. Application developers indirectly benefit through `quoin`‑based libraries.

**Diagram Description (textual):**
- **System:** `quoin` (Core + Adapters)
- **Actors:**
  - **Library Author:** Consumes `quoin` core and selects an adapter.
  - **Application Developer:** Uses libraries built on `quoin`.
- **External Systems:**
  - **GPUI, Dioxus, Leptos, Xilem, Floem:** Target UI frameworks that provide native reactive primitives.

### 5.2 Container Diagram (C2)

| Container | Technology | Description | Responsibilities |
|-----------|------------|-------------|------------------|
| **quoin** | Rust Library | Core crate defining `ReactiveContext`, `Signal`, `Executor` traits. | Provide the abstraction contract. |
| **quoin‑gpui** | Rust Library | Adapter implementing traits for GPUI. | Bridge `quoin` to GPUI's `Entity` and `Context`. |
| **quoin‑dioxus** | Rust Library | Adapter implementing traits for Dioxus. | Bridge `quoin` to Dioxus's `Signal` and `Scope`. |
| **quoin‑leptos** | Rust Library | Adapter implementing traits for Leptos. | Bridge `quoin` to Leptos's `RwSignal` and `create_effect`. |
| **quoin‑xilem** | Rust Library | Adapter implementing traits for Xilem. | Bridge `quoin` to Xilem's `View` and state management. |
| **quoin‑floem** | Rust Library | Adapter implementing traits for Floem. | Bridge `quoin` to Floem's `RwSignal` and `ViewId`. |
| **quoin‑conformance** | Rust Test Crate | Shared test suite for validating adapters. | Ensure all adapters meet the contract. |

**Relationships:**
- All adapters depend on `quoin` (core).
- `quoin‑conformance` depends on `quoin` and is used by each adapter's test suite.
- Downstream libraries depend on `quoin` and exactly one adapter (via feature flags).

### 5.3 Component Diagram (C3) — Core Crate Internals

**Components within `quoin` core:**
- **`reactive` module:** Contains `ReactiveContext` trait definition.
- **`signal` module:** Contains `Signal<T>` and optionally `MutableSignal<T>` traits.
- **`executor` module:** Contains `Executor` trait and `CancellationToken` utility.
- **`lib.rs`:** Re‑exports public API and defines feature flag logic.

**Relationships:**
- `executor` and `signal` are independent but referenced by `ReactiveContext`.
- `CancellationToken` is a standalone utility used by downstream libraries.

---

## 6. Architecture Decision Records (ADRs)

### ADR‑001: Trait‑Based Reactive Abstraction with Feature Flags

**Status:** Accepted  
**Date:** 2026‑04‑19  
**Context:** The Rust UI ecosystem lacks a shared reactive foundation. `quoin` must provide a way for libraries to be framework‑agnostic without runtime overhead.

**Decision Drivers:**
- ASR‑001: Zero‑Cost Abstraction
- ASR‑002: Compile‑Time Framework Selection
- ASR‑004: Minimal Core Dependencies

**Considered Options:**
1. **Macro‑based code generation:** Downstream libraries use macros to generate framework‑specific code at compile time.
   - *Pros:* Very flexible; can generate optimal code.
   - *Cons:* Complex to maintain; poor IDE support; harder to document.
2. **Runtime trait objects with dynamic dispatch:** Core traits use `dyn Trait` and adapters are selected at runtime.
   - *Pros:* Simple to implement; enables runtime switching.
   - *Cons:* Violates ASR‑001 (runtime overhead); violates Vision Non‑Goal of no runtime switching.
3. **Trait‑based abstraction with feature flags:** Core defines traits; adapters implement them; feature flags select a single adapter at compile time.
   - *Pros:* Zero‑cost (static dispatch); enforces single‑framework constraint; idiomatic Rust.
   - *Cons:* Requires discipline to avoid accidentally depending on multiple adapters.

**Decision Outcome:** Chose **Option 3 – Trait‑based abstraction with feature flags**.

**Consequences:**
- *Positive:* Meets all ASRs; uses standard Rust patterns; excellent performance.
- *Negative:* Downstream libraries must carefully manage feature flags; cross‑adapter code sharing is not possible.
- *Follow‑up:* Implement compile‑time checks to prevent multiple adapter features from being enabled simultaneously.

### ADR‑002: Signal Trait Design – Static Dispatch vs. Dynamic Dispatch

**Status:** Accepted  
**Date:** 2026‑04‑19  
**Context:** The `Signal<T>` trait must be `Clone + Copy` to be easily passed into closures. The implementation must support both framework‑native signal types (e.g., Leptos's `RwSignal<T>`) and derived signals.

**Decision Drivers:**
- ASR‑001: Zero‑Cost Abstraction
- ASR‑006: Adapter Implementation Complexity ≤ 200 LOC

**Considered Options:**
1. **Wrapper struct with internal `dyn SignalTrait`:** A concrete `Signal<T>` struct that type‑erases the underlying framework signal.
   - *Pros:* Single type; easy to store in collections.
   - *Cons:* Dynamic dispatch overhead; violates ASR‑001.
2. **Pure trait with associated types:** Each adapter defines its own concrete signal type that implements `Signal<T>`.
   - *Pros:* Zero‑cost static dispatch; fully flexible.
   - *Cons:* Downstream libraries must be generic over `S: Signal<T>`; more complex API.
3. **Hybrid: Trait + optional type‑erased wrapper:** Core defines `Signal<T>` trait; an optional `AnySignal<T>` wrapper provides dynamic dispatch for use cases that need it.
   - *Pros:* Best of both worlds; static dispatch by default; dynamic dispatch opt‑in.
   - *Cons:* Slightly larger API surface.

**Decision Outcome:** Chose **Option 3 – Hybrid trait + optional wrapper**.

**Consequences:**
- *Positive:* Satisfies zero‑cost requirement for most use cases; provides escape hatch for dynamic dispatch.
- *Negative:* Adds an extra type (`AnySignal<T>`); documentation must guide users on when to use each.

### ADR‑003: Executor Abstraction and Cancellation Strategy

**Status:** Accepted  
**Date:** 2026‑04‑19  
**Context:** `quoin` must abstract over different async runtimes: Tokio, `wasm‑bindgen‑futures`, and GPUI's foreground executor. Cancellation behavior varies across runtimes.

**Decision Drivers:**
- ASR‑003: Async Runtime Neutrality
- ASR‑006: Adapter Implementation Complexity ≤ 200 LOC

**Considered Options:**
1. **Full executor abstraction with cancellation:** `Executor` trait includes methods for spawning, cancellation, and task locals.
   - *Pros:* Comprehensive.
   - *Cons:* High implementation burden; leaky abstraction due to runtime differences.
2. **Minimal `spawn` only; separate `CancellationToken` utility:** `Executor` only has `spawn`. Cancellation is cooperatively handled via a provided `CancellationToken` type.
   - *Pros:* Simple to implement; works across all runtimes; cancellation is explicit and portable.
   - *Cons:* Downstream libraries must manually wire cancellation tokens.
3. **No executor abstraction; rely on `futures::executor`:** Downstream libraries use existing executor crates directly.
   - *Pros:* Zero new code.
   - *Cons:* Does not solve framework‑agnosticism; GPUI executor is not standard.

**Decision Outcome:** Chose **Option 2 – Minimal `spawn` + `CancellationToken` utility**.

**Consequences:**
- *Positive:* Meets ASR‑003; adapters are trivial to write.
- *Negative:* Libraries must use cooperative cancellation patterns; `JoinHandle` API may differ slightly across runtimes (mitigated by `quoin`'s `JoinHandle` wrapper).

### ADR‑004: Conformance Test Suite Using `tested‑trait`

**Status:** Accepted  
**Date:** 2026‑04‑19  
**Context:** Adapters must be validated to ensure consistent behavior. The test suite must be reusable by any adapter crate.

**Decision Drivers:**
- ASR‑005: Adapter Conformance Test Pass Rate

**Considered Options:**
1. **Manual test replication:** Each adapter copies the same test file and adapts it.
   - *Pros:* No extra dependencies.
   - *Cons:* High maintenance; tests diverge.
2. **Custom macro that generates tests:** A procedural macro that expands to framework‑specific tests.
   - *Pros:* Centralized test logic.
   - *Cons:* Complex macro; harder to debug.
3. **Use `tested‑trait` crate:** This crate allows defining tests within a trait definition and running them against any implementor.
   - *Pros:* Clean separation; tests are defined once; adapters simply use `#[test_impl]`.
   - *Cons:* Adds a dev‑dependency on `tested‑trait`.

**Decision Outcome:** Chose **Option 3 – `tested‑trait`**.

**Consequences:**
- *Positive:* Single source of truth for conformance; easy for adapter authors.
- *Negative:* Introduces a build dependency for testing.

### ADR‑005: Adapter Crate Organization

**Status:** Accepted  
**Date:** 2026‑04‑19  
**Context:** Adapters need to be discoverable and maintainable. The "All Adapters Are Equal" policy must be upheld.

**Decision Drivers:**
- ASR‑002: Feature‑Flagged Adapter Selection
- BRS Policy: All Adapters Are Equal

**Considered Options:**
1. **Monorepo with all adapters in the `quoin` repository.**
   - *Pros:* Centralized control; easier CI.
   - *Cons:* High maintenance burden on core team; discourages community adapters.
2. **Independent crates published separately; listed in documentation.**
   - *Pros:* Decentralized; community‑driven; aligns with "All Adapters Are Equal".
   - *Cons:* Harder to enforce conformance (mitigated by test suite).
3. **Hybrid: Core adapters in monorepo; community adapters independent.**
   - *Pros:* Balance of control and community.
   - *Cons:* Creates a two‑tier system; violates "All Adapters Are Equal".

**Decision Outcome:** Chose **Option 2 – Independent crates**.

**Consequences:**
- *Positive:* Encourages community contributions; reduces core team maintenance burden.
- *Negative:* Core team must provide clear guidance and a conformance badge system.
- *Follow‑up:* Create a `quoin‑adapters` GitHub repository that serves as an index of known adapters (not a monorepo).

---

## 7. API and Interface Contracts

### 7.1 `ReactiveContext` Trait

```rust
// In quoin core crate
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    /// The framework's native signal type.
    type Signal<T: 'static>: Signal<T>;

    /// The framework's async executor.
    type Executor: Executor;

    /// Create a new signal with an initial value.
    fn create_signal<T: 'static>(&self, initial: T) -> Self::Signal<T>;

    /// Get the executor for spawning tasks.
    fn executor(&self) -> Self::Executor;

    /// Request that the UI re‑render.
    /// For frameworks with automatic reactivity, this may be a no‑op.
    fn request_update(&self);
}
```

### 7.2 `Signal<T>` Trait

```rust
pub trait Signal<T: 'static>: Clone + Copy {
    /// Get the current value.
    fn get(&self) -> T;

    /// Read the value with a closure, avoiding a clone.
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U;
}

// Optional extension trait
pub trait MutableSignal<T: 'static>: Signal<T> {
    fn set(&self, value: T);
    fn update(&self, f: impl FnOnce(&mut T));
}

// Optional type‑erased wrapper
pub struct AnySignal<T> { /* ... */ }
impl<T> Signal<T> for AnySignal<T> { /* ... */ }
```

### 7.3 `Executor` Trait

```rust
pub trait Executor: Clone + Send + Sync + 'static {
    type JoinHandle<T: Send + 'static>: JoinHandle<T>;

    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

pub trait JoinHandle<T> {
    fn abort(&self);
    // ... other methods as needed
}

// Utility for cooperative cancellation
pub struct CancellationToken { /* ... */ }
impl CancellationToken {
    pub fn new() -> Self;
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
    pub fn cancelled(&self) -> impl Future<Output = ()>;
}
```

---

## 8. Cross‑Cutting Concerns

### 8.1 Observability
- The `quoin` core does not provide built‑in logging or metrics. Adapters MAY instrument their implementations with `tracing` spans.
- Future `quoin‑devtools` crate will provide a message bus for observing signal updates and task lifecycle.

### 8.2 Error Handling
- The core traits are infallible by design. Framework adapters must handle any internal errors gracefully (e.g., panic on unrecoverable errors, or log and recover).
- The `Executor::spawn` method returns a `JoinHandle` that can be used to await task completion and handle panics.

### 8.3 Documentation Generation
- All public API items must have `///` doc comments with examples.
- The `quoin` website (future) will host the rendered documentation and an adapter index.

---

## 9. Deployment and Build Considerations

### 9.1 Crate Features
**Core crate (`quoin`):**
- No default features.
- Optional `macros` feature (future) for proc‑macro helpers.
- Optional `devtools` feature (future) for instrumentation.

**Adapter crates (e.g., `quoin‑gpui`):**
- Declare `quoin` as a dependency.
- Re‑export a concrete `ReactiveContext` type (e.g., `GpuiContext`).

**Downstream library:**
```toml
[dependencies]
quoin = "1.0"

[features]
gpui = ["quoin‑gpui"]
dioxus = ["quoin‑dioxus"]
leptos = ["quoin‑leptos"]
```

### 9.2 Compile‑Time Checks
The core crate provides a compile‑time assertion that fails if multiple adapter features are enabled simultaneously:
```rust
#[cfg(any(
    all(feature = "gpui", feature = "dioxus"),
    all(feature = "gpui", feature = "leptos"),
    // ... all pairwise combinations
))]
compile_error!("Only one framework adapter feature may be enabled at a time.");
```

### 9.3 Conformance Testing in CI
Each adapter crate runs the conformance test suite in its CI pipeline. A passing conformance test is a prerequisite for being listed in the official adapter index.

---

## 10. Alternatives Considered

This section summarizes key alternatives that were evaluated and rejected. Detailed trade‑offs are captured in the ADRs.

| Decision Area | Rejected Alternative | Reason |
|---------------|----------------------|--------|
| Abstraction Mechanism | Macro‑based code generation | Too complex; poor maintainability. |
| Signal Type | Pure trait without wrapper | API complexity for dynamic use cases. |
| Executor Abstraction | Full cancellation support | Leaky abstraction; high adapter complexity. |
| Conformance Testing | Manual test replication | High maintenance; risk of divergence. |
| Adapter Organization | Monorepo | High core team burden; discourages community. |

---

## 11. Traceability: ASRs to Design Decisions

| ASR ID | Addressed by ADR(s) | Addressed by Component(s) |
|--------|---------------------|---------------------------|
| ASR‑001 (Zero‑Cost) | ADR‑001, ADR‑002 | Core traits use static dispatch; feature flags enable monomorphization. |
| ASR‑002 (Feature Flags) | ADR‑001, ADR‑005 | `Cargo.toml` feature definitions; compile‑time mutual exclusion. |
| ASR‑003 (Async Neutrality) | ADR‑003 | `Executor` trait; `CancellationToken` utility. |
| ASR‑004 (Minimal Dependencies) | ADR‑001 | `Cargo.toml` only depends on `futures‑core`. |
| ASR‑005 (Conformance Tests) | ADR‑004 | `quoin‑conformance` crate using `tested‑trait`. |
| ASR‑006 (Adapter LOC) | ADR‑002, ADR‑003 | Minimal trait surface; reference adapters demonstrate low LOC. |

---

## 12. Risks and Open Issues

| ID | Description | Mitigation | Status |
|----|-------------|------------|--------|
| **RISK‑ARCH‑01** | `tested‑trait` may not be mature enough for production use. | Evaluate alternatives (e.g., custom macro) if issues arise. | Monitor |
| **RISK‑ARCH‑02** | GPUI's executor model may be incompatible with `futures::Future`. | Investigate GPUI's `cx.spawn()` API; may require a custom wrapper. | Open |
| **TBD‑ARCH‑01** | Finalize the `JoinHandle` API to be compatible across Tokio and WASM. | Core Team | Design Phase |
| **TBD‑ARCH‑02** | Determine whether `MutableSignal` should be part of core or an extension trait. | Core Team | Design Phase |
