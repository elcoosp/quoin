
## 1. Product Vision & Strategic Alignment (`shadcn-rs-api-alignment-vision.md`)

```markdown
# Shadcn UI for Rust – API Alignment Vision

| Field | Value |
|-------|-------|
| Project | Shadcn UI API Alignment for Quoin UCP |
| Document | Vision & Strategic Alignment |
| Version | 0.1 (Draft) |
| Date | 2026-04-21 |
| Author | System Architect (assisted by AI) |
| Status | Draft — Pending Review |

## Vision Statement

To align the public APIs of the existing **Leptos‑shadcn/ui** and **Dioxus‑shadcn/ui** component libraries under a common, framework‑agnostic specification, enabling Quoin’s Universal Component Protocol (UCP) to target both as consistent renderer backends without maintaining bespoke adapters.

## Elevator Pitch (Moore’s Template)

**For** Quoin UCP developers and maintainers of Rust shadcn/ui ports  
**who are frustrated** by the fragmented APIs and uneven component coverage between Leptos‑shadcn/ui and Dioxus‑shadcn/ui,  
**our initiative** defines a common API specification and conformance test suite  
**that provides** a clear, achievable target for both libraries to converge toward.  
**Unlike** ad‑hoc integration efforts,  
**our approach** treats the existing libraries as first‑class implementations and focuses on incremental alignment, reducing Quoin’s renderer complexity and benefiting the entire Rust UI ecosystem.

## Problem Statement & Business Context

**Current Reality:**
- **Leptos‑shadcn/ui** (~50 components) and **Dioxus‑shadcn/ui** (~26 components) are valuable, production‑ready libraries.
- Their APIs differ significantly: component names, prop names, event handler signatures, and feature depth (validation, accessibility, complex components).
- Quoin’s UCP must currently maintain two divergent backend mappings to support both frameworks, increasing maintenance cost and limiting component coverage.

**Opportunity:** By defining a **common API specification** that both libraries agree to target, Quoin can reduce its renderer logic to a single mapping. The libraries themselves benefit from clear guidance on feature parity and a shared conformance test suite.

## Target Users & Customers

**Primary:**
- **Quoin UCP Contributors** – Need a stable, predictable component contract across Leptos and Dioxus.
- **Leptos‑shadcn/ui Maintainers** – Seek to expand component coverage and improve consistency with community standards.
- **Dioxus‑shadcn/ui Maintainer** – Aims to close the feature gap with the Leptos port.

**Secondary:**
- **Rust Frontend Developers** – Benefit indirectly from improved consistency and component availability.

## User Needs & Value Proposition

| Need | Description |
|------|-------------|
| **API Consistency** | Identical component names, prop names, and event handler signatures across both libraries. |
| **Component Parity** | All shadcn/ui components available in both implementations. |
| **Clear Alignment Path** | A specification that defines the target API for each component, allowing incremental convergence. |
| **Conformance Validation** | A shared test suite that verifies both implementations against the spec. |

## Desired Outcomes & Success Metrics

| ID | Outcome | Key Results |
|----|---------|-------------|
| **G‑1** | **API Convergence** | 100% of components in the common spec have identical prop names and event signatures in both libraries. |
| **G‑2** | **Component Parity** | 100% of components in the common spec are implemented in both libraries. |
| **G‑3** | **Quoin Integration** | Quoin’s `quoin_render!` macro uses a single mapping to target both libraries, with zero framework‑specific conditionals per component. |
| **G‑4** | **Conformance Pass Rate** | Both libraries pass the shared conformance test suite with 100% success. |

## Strategic Constraints

- **No Rewrites:** The existing libraries remain the canonical implementations; changes are incremental and backward‑compatible where possible.
- **Framework‑Agnostic Spec:** The specification defines logical props (e.g., `disabled`) without dictating concrete Rust types. Each library adapts to its native reactivity model (Leptos `MaybeProp<T>` or `(ReadSignal, WriteSignal)`, Dioxus `Signal<T>`).
- **WASM Compatibility:** All components must continue to work in WebAssembly.
- **Accessibility:** WCAG 2.1 AA compliance is a non‑negotiable quality requirement.

## Goals and Non‑goals

**Goals:**
- Define a common API specification (component names, props, events, ARIA roles) for all 50+ shadcn/ui components.
- Provide a conformance test suite that validates both existing libraries against the spec.
- Enable Quoin’s UCP to use a unified mapping to both libraries.

**Non‑goals:**
- Creating a new shadcn/ui implementation from scratch.
- Changing the visual design or CSS class names of existing components.
- Unifying the internal architecture of the two libraries (they remain independent).

## Operational Concept & High‑Level Scenarios

**Scenario A: Quoin UCP Rendering**
1. A developer writes a Quoin component using `quoin_render!`.
2. The macro consults the common API spec to generate framework‑specific code.
3. With `features = ["leptos"]`, it emits `leptos_shadcn_button::Button` with appropriate reactive bindings. With `features = ["dioxus"]`, it emits `dioxus_shadcn_button::Button` with the same logical props.

**Scenario B: Library Maintainer Adding a Component**
1. The maintainer consults the common spec to see the required props and events for a new component (e.g., `Combobox`).
2. They implement the component in their library, following the spec’s naming and behavior.
3. They run the conformance test suite locally to validate their implementation.

## Stakeholders, Sponsorship, and Governance

| Role | Responsible |
|------|-------------|
| **Executive Sponsors** | Core maintainers of Leptos‑shadcn/ui and Dioxus‑shadcn/ui |
| **Spec Editor** | Designated community member (rotating) |
| **Quoin Maintainer** | Provides UCP integration requirements |
| **Decision Model** | Changes to the spec require approval from at least one maintainer from each library. |

## Traceability & Alignment Notes

- All components assigned unique ID (e.g., `SPEC‑BUTTON‑001`).
- The conformance test suite maps spec requirements to automated DOM checks.

## Risks, Assumptions, and Open Questions

**Assumptions:**
- Both library maintainers are willing to align their APIs incrementally.
- Backward compatibility can be maintained during the transition (e.g., deprecation cycles).

**Risks:**
- **Divergent Reactivity Models:** Prop types will necessarily differ (Leptos `MaybeProp<T>`, Dioxus `Signal<T>`). The spec handles this by defining logical prop semantics; the conformance suite validates behavior, not types.
- **Maintainer Bandwidth:** Alignment work may be slow due to limited volunteer time.

**Open Questions:**
- Q1: How to handle components that exist in only one library today (e.g., `InputGroup` in Dioxus, `Select` in Leptos)? The spec should include them, and the missing library should implement them.
- Q2: What is the appropriate deprecation and migration timeline for breaking API changes?
```

---

## 2. Business & Stakeholder Requirements Specification (`shadcn-rs-api-alignment-brs.md`)

```markdown
# Business & Stakeholder Requirements Specification (BRS)

| Field | Value |
|-------|-------|
| Project | Shadcn UI API Alignment for Quoin UCP |
| Document | Business & Stakeholder Requirements |
| Version | 0.1 (Draft) |
| Date | 2026-04-21 |
| Author | System Architect (assisted by AI) |
| Status | Draft — Pending Review |

## 1. Business Context

### 1.1 Purpose
This BRS defines the business and stakeholder requirements for aligning the public APIs of **Leptos‑shadcn/ui** and **Dioxus‑shadcn/ui** under a common specification. It captures the needs of Quoin UCP developers and the library maintainers, establishing a shared understanding of the desired outcome.

### 1.2 Business Problem/Opportunity
The two leading Rust shadcn/ui implementations have diverged in component inventory, prop naming, and event signatures. This forces Quoin’s UCP to maintain two separate renderer backends and creates confusion for developers working across frameworks. Aligning the APIs reduces maintenance cost, improves component coverage, and strengthens the Rust UI ecosystem.

### 1.3 Scope Boundaries
**In‑Scope:**
- A common API specification (component names, prop names, event names, ARIA roles) for all 50+ shadcn/ui components.
- A shared conformance test suite that validates both existing libraries against the spec.
- Incremental updates to the existing libraries to achieve conformance.

**Out‑of‑Scope:**
- Creating a new shadcn/ui implementation.
- Changing the visual styling or CSS architecture of the libraries.
- Unifying the internal codebases of the two libraries.

## 2. Business Goals, Objectives & Success Metrics

| ID | Goal | Fit Criterion | Priority |
|----|------|---------------|----------|
| **BG‑1** | API Convergence | 100% of components in the common spec have identical prop names and event signatures in both libraries. | High |
| **BG‑2** | Component Parity | 100% of components in the common spec are implemented in both libraries. | High |
| **BG‑3** | Quoin UCP Simplification | Quoin’s `quoin_render!` macro reduces framework‑specific code by at least 80% (measured by lines of emitter code). | High |
| **BG‑4** | Conformance Validation | Both libraries pass the shared conformance test suite with 100% success. | High |

## 3. Business Model and Processes

### 3.1 Value Propositions
- **For Quoin:** Single, predictable component contract; eliminates per‑framework conditional logic.
- **For Library Maintainers:** Clear roadmap for component parity; shared testing infrastructure reduces QA burden.
- **For Developers:** Consistent shadcn/ui experience across Leptos and Dioxus.

### 3.2 Core Business Processes
1. **Specification Maintenance:** The common spec is hosted in a dedicated repository. Changes are proposed via PR and reviewed by maintainers from both libraries.
2. **Incremental Alignment:** Each library maintainer prioritizes work to align existing components and add missing ones, following the spec.
3. **Conformance Testing:** A shared test suite runs in CI for both libraries. A dashboard displays compliance status.
4. **Quoin Integration:** Quoin’s UCP renderer uses the spec as its source of truth for component APIs.

## 4. Business Rules and Policies

| ID | Rule | Description |
|----|------|-------------|
| **BR‑001** | Backward Compatibility | Breaking API changes must follow a deprecation cycle (minimum one minor version) with clear migration guidance. |
| **BR‑002** | Accessibility Mandate | All components must meet WCAG 2.1 AA standards, as verified by the conformance suite. |
| **BR‑003** | Spec‑First for New Components | Any new component added to one library must be added to the common spec first, then implemented in both libraries. |

## 5. Stakeholders and User Classes

### 5.1 Stakeholder Map
| Stakeholder | Primary Concerns |
|-------------|------------------|
| Quoin UCP Maintainers | Consistent component contract; minimal framework‑specific code. |
| Leptos‑shadcn/ui Maintainers | Preserving quality; managing breaking changes; community expectations. |
| Dioxus‑shadcn/ui Maintainer | Closing feature gap; aligning with community standards. |
| Rust Frontend Developers | Consistent, well‑documented components across frameworks. |

### 5.2 User Classes and Personas
**Primary: Quoin UCP Developer (“Quinn”)**
- Maintains Quoin’s renderer backends. Quinn wants the two shadcn/ui libraries to expose identical prop names and event signatures so that `quoin_render!` can generate code without framework‑specific conditionals.

**Secondary: Library Maintainer (“Lee”)**
- Maintains one of the shadcn/ui ports. Lee wants clear guidance on what components to add and how to name their props, plus a test suite to validate correctness.

### 5.3 Jobs to Be Done (JTBD)
- **When** implementing a Quoin renderer, **I want to** map a logical component (e.g., “Button”) to a concrete library function with predictable prop names **so that** I don’t write framework‑specific adapters.
- **When** adding a new shadcn/ui component, **I want to** follow a standard API definition **so that** my implementation is consistent with the other framework’s port.

## 6. Glossary / Ubiquitous Language

| Term | Definition |
|------|------------|
| **Common Spec** | Framework‑agnostic definition of a component’s public API (props, events, ARIA roles). |
| **Conformance Test** | Automated test verifying that a library’s component produces the expected DOM structure and behavior. |
| **Logical Prop** | A prop defined in the spec without a concrete Rust type (e.g., `disabled`). Each library maps it to its native reactive type. |

## 7. Conceptual Domain Model

**Entities:**
- **ComponentSpec:** Name, props (with reactivity flag), events, ARIA roles.
- **LibraryImpl:** Leptos‑shadcn/ui or Dioxus‑shadcn/ui.
- **ConformanceTest:** Validates a LibraryImpl against a ComponentSpec.

**Relationships:**
- A `ComponentSpec` is implemented by up to two `LibraryImpl`s.
- A `LibraryImpl` is validated by 1..* `ConformanceTest`s.

## 8. Stakeholder Needs and User Requirements

| ID | Need | User Class | Priority |
|----|------|------------|----------|
| **SN‑001** | Identical component names, prop names, and event names across both libraries. | Quinn, Lee | High |
| **SN‑002** | All shadcn/ui components available in both libraries. | Quinn, Lee | High |
| **SN‑003** | A conformance test suite to validate implementations. | Lee | High |
| **SN‑004** | Clear documentation of the common API for each component. | Quinn, Lee | High |

## 9. System‑in‑Context and Operational Concept

**System‑of‑Interest:** Common API Specification + Conformance Test Suite + Updated Leptos/Dioxus Libraries.

**Operational Concept:**
1. Quoin’s `quoin_render!` macro consults the common spec to generate calls to the appropriate library.
2. A library maintainer implements a new component following the spec; they run the conformance tests locally to verify.
3. CI runs the conformance suite against both libraries on every PR, ensuring ongoing alignment.

## 10. Stakeholder‑Level Constraints and Quality Expectations

| ID | Constraint / Quality Expectation | Fit Criterion |
|----|----------------------------------|---------------|
| **CON‑001** | No rewrites of existing libraries | Changes are incremental, via normal PR processes. |
| **CON‑002** | Backward compatibility | Breaking changes follow deprecation cycle. |
| **QUAL‑001** | Conformance test pass rate | 100% for both libraries. |

## 11. Risks, Assumptions, and Open Issues

*(See Vision document.)*

## 12. Traceability Mapping to Vision

| Vision Goal | Stakeholder Need | High‑Level Feature |
|-------------|------------------|-------------------|
| G‑1 (API Convergence) | SN‑001 | Common API spec |
| G‑2 (Component Parity) | SN‑002 | Component catalog alignment |
| G‑3 (Quoin Integration) | SN‑004 | Unified UCP mapping |
| G‑4 (Conformance) | SN‑003 | Shared test suite |
```

---

## 3. Software Requirements Specification (`shadcn-rs-api-alignment-srs.md`)

```markdown
# Software Requirements Specification (SRS)

| Field | Value |
|-------|-------|
| Project | Shadcn UI API Alignment for Quoin UCP |
| Document | Software Requirements Specification |
| Version | 0.1 (Draft) |
| Date | 2026-04-21 |
| Author | System Architect (assisted by AI) |
| Status | Draft — Pending Review |

## 1. Introduction and Scope

### 1.1 Purpose
This SRS defines the functional and non‑functional requirements for:
1. The **Common API Specification** for shadcn/ui components.
2. The **Conformance Test Suite** that validates Leptos‑shadcn/ui and Dioxus‑shadcn/ui against the spec.
3. The required updates to the existing libraries to achieve conformance.

### 1.2 Scope
- All 50+ shadcn/ui components.
- Public API surface: component names, prop names, event names, ARIA roles.
- Behavioral consistency (DOM output, event handling, validation).

**Out‑of‑Scope:**
- Internal implementation details of the libraries.
- Visual styling or CSS class names.

## 2. System Context and Overview

The system comprises:
- **Common API Specification:** JSON schemas defining component APIs.
- **Leptos‑shadcn/ui** and **Dioxus‑shadcn/ui** – existing libraries that will be updated to conform.
- **Conformance Test Suite:** WebDriver‑based tests validating DOM output against the spec.

## 3. Functional Capabilities and Behavior

### 3.1 Capability: Common API Specification

| ID | Requirement (EARS Pattern) | Priority |
|----|----------------------------|----------|
| **F‑SPEC‑001** | The common spec shall define a unique identifier for each component (e.g., `"button"`). | Must |
| **F‑SPEC‑002** | For each component, the spec shall list all props, including name, description, and whether the prop is reactive. | Must |
| **F‑SPEC‑003** | The spec shall define supported events, including name and payload type. | Must |
| **F‑SPEC‑004** | The spec shall define expected ARIA roles, states, and properties. | Must |
| **F‑SPEC‑005** | The spec shall be stored in a machine‑readable format (JSON Schema) to enable tooling. | Must |

### 3.2 Capability: Library API Alignment

| ID | Requirement | Priority |
|----|-------------|----------|
| **F‑LIB‑001** | The Leptos‑shadcn/ui library shall expose components with names matching the common spec. | Must |
| **F‑LIB‑002** | The Dioxus‑shadcn/ui library shall expose components with names matching the common spec. | Must |
| **F‑LIB‑003** | Prop names in both libraries shall match the common spec. | Must |
| **F‑LIB‑004** | Event handler names shall match the common spec (e.g., `on_click` in Leptos, `onclick` in Dioxus; the spec defines the logical name, and each library uses its idiomatic casing). | Must |
| **F‑LIB‑005** | If a component exists in only one library, the missing library shall implement it within two release cycles. | Should |

### 3.3 Capability: Behavioral Consistency

| ID | Requirement | Priority |
|----|-------------|----------|
| **F‑BEH‑001** | For equivalent props, the generated DOM structure (element types, class names, ARIA attributes) shall be identical in both libraries. | Must |
| **F‑BEH‑002** | Event handling semantics (when events fire, payload structure) shall be identical. | Must |
| **F‑BEH‑003** | Validation rules and error display shall be identical. | Should |

### 3.4 Capability: Conformance Test Suite

| ID | Requirement | Priority |
|----|-------------|----------|
| **F‑TEST‑001** | A shared test suite shall be developed that can be executed against both libraries. | Must |
| **F‑TEST‑002** | The test suite shall cover at least 90% of behavioral requirements. | Should |
| **F‑TEST‑003** | Test results shall be published as a dashboard showing per‑component compliance. | Should |

## 4. Quality and Non‑functional Requirements

| ID | Category | Requirement | Fit Criterion |
|----|----------|-------------|---------------|
| **NFR‑MAIN‑001** | Maintainability | The common spec shall be versioned semantically. | Major version changes indicate breaking API changes. |
| **NFR‑COMP‑001** | Compatibility | Libraries shall continue to support their minimum Rust version and WASM target. | Existing CI passes. |

## 5. External Interfaces and Data Contracts

### 5.1 Common Spec Format (Example: Button)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://shadcn-rs.org/spec/button.json",
  "title": "Button",
  "props": {
    "variant": {
      "type": "string",
      "enum": ["default", "destructive", "outline", "secondary", "ghost", "link"],
      "default": "default",
      "reactive": true
    },
    "size": {
      "type": "string",
      "enum": ["default", "sm", "lg", "icon"],
      "default": "default",
      "reactive": true
    },
    "disabled": { "type": "boolean", "default": false, "reactive": true },
    "loading": { "type": "boolean", "default": false, "reactive": true }
  },
  "events": {
    "onClick": { "description": "Fired when the button is clicked." }
  },
  "aria": { "role": "button", "attributes": ["aria-disabled", "aria-busy"] }
}
```

### 5.2 Library API Examples (Conforming)

**Leptos‑shadcn/ui (target):**
```rust
#[component]
pub fn Button(
    #[prop(into, optional)] variant: MaybeProp<ButtonVariant>,
    #[prop(into, optional)] size: MaybeProp<ButtonSize>,
    #[prop(into, optional)] disabled: MaybeProp<bool>,
    #[prop(into, optional)] loading: MaybeProp<bool>,
    #[prop(optional)] on_click: Option<Callback<web_sys::MouseEvent>>,
    children: Children,
) -> impl IntoView
```

**Dioxus‑shadcn/ui (target):**
```rust
#[component]
pub fn Button(
    variant: Option<ButtonVariant>,
    size: Option<ButtonSize>,
    disabled: Option<bool>,
    loading: Option<bool>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element
```

## 6. Constraints, Assumptions, and Dependencies

- **C‑001:** Leptos library uses Leptos v0.8+.
- **C‑002:** Dioxus library uses Dioxus v0.7+.
- **A‑001:** Both library maintainers agree to align with the common spec.
- **D‑001:** Conformance test suite depends on `fantoccini` and `wasm-bindgen-test`.

## 7. TBD Log

| ID | Item | Owner |
|----|------|-------|
| TBD‑001 | Finalize JSON Schema for all components. | Spec Editor |
| TBD‑002 | Define deprecation and migration plan for breaking changes. | Maintainers |

## 8. Requirements Attributes and Traceability Model

- **ID Scheme:** `F‑{CATEGORY}‑{NNN}`.
- Each requirement traces to a Stakeholder Need (SN‑xxx) from the BRS.
```

---

## 4. Architecture & Design Specification (`shadcn-rs-api-alignment-architecture.md`)

```markdown
# Architecture & Design Specification

| Field | Value |
|-------|-------|
| Project | Shadcn UI API Alignment for Quoin UCP |
| Document | Architecture & Design Specification |
| Version | 0.1 (Draft) |
| Date | 2026-04-21 |
| Author | System Architect (assisted by AI) |
| Status | Draft — Pending Review |

## 1. Context and Scope

This document describes the architectural approach for aligning the existing Leptos‑shadcn/ui and Dioxus‑shadcn/ui libraries under a common API specification, without rewriting them.

**Goals:**
- Define a machine‑readable common spec that serves as the source of truth for component APIs.
- Provide a conformance test suite that validates the existing libraries against the spec.
- Enable Quoin’s UCP to use a unified mapping to both libraries.

**Non‑goals:**
- Changing the internal architecture of the libraries.
- Creating a new shared codebase.

## 2. Architecturally Significant Requirements (ASRs)

| ID | Requirement | Impact |
|----|-------------|--------|
| **ASR‑001** | Common spec must be machine‑readable. | JSON Schema chosen. |
| **ASR‑002** | Conformance tests must work against both libraries without modification. | WebDriver/DOM‑level testing. |
| **ASR‑003** | Libraries must be able to evolve independently while tracking the spec. | Spec versioned; libraries reference spec version. |

## 3. System Overview

```
┌─────────────────────────┐
│  Common API Spec (JSON) │
└───────────┬─────────────┘
            │ (references)
            ▼
┌─────────────────────────┐      ┌─────────────────────────┐
│ Leptos‑shadcn/ui        │      │ Dioxus‑shadcn/ui        │
│ (existing, updated)     │      │ (existing, updated)     │
└───────────┬─────────────┘      └───────────┬─────────────┘
            │                                │
            └──────────────┬─────────────────┘
                           │ (test against)
                           ▼
                  ┌─────────────────┐
                  │ Conformance     │
                  │ Test Suite      │
                  └─────────────────┘
```

## 4. Common API Specification Repository

**Structure:**
```
shadcn-rs-spec/
├── schemas/
│   ├── button.json
│   ├── input.json
│   └── ...
├── docs/
│   └── (generated markdown)
└── tests/
    └── conformance/
        ├── scenarios/       # YAML test scenarios
        └── runner/          # Rust WebDriver runner
```

**Versioning:** The spec uses semantic versioning. Libraries declare which spec version they conform to (e.g., in `Cargo.toml` metadata or documentation).

## 5. Library Updates (Incremental)

Both libraries continue to live in their existing repositories. Changes are made via normal PR processes:

- **Renaming props/events** to match the spec (with deprecation cycle if breaking).
- **Adding missing components** following the spec.
- **Running conformance tests** in CI to verify compliance.

**Deprecation Strategy:**
1. Introduce new API (matching spec) alongside old API.
2. Mark old API as `#[deprecated]`.
3. Remove old API in next major version.

## 6. Conformance Test Suite

Uses `fantoccini` (Rust WebDriver) to drive a headless browser. For each component:
1. Mount the component with a predefined set of props (using a minimal test harness for each library).
2. Query the DOM for expected elements, classes, and ARIA attributes.
3. Simulate interactions and verify callbacks.
4. Compare against expected DOM snapshot.

Test scenarios are defined in YAML, framework‑agnostic.

## 7. Quoin UCP Integration

Quoin’s `quoin_render!` macro uses the common spec as its source of truth. The macro’s emitters generate code that calls the appropriate library functions. For example:

**Leptos emitter:**
```rust
leptos_shadcn_button::Button(
    variant: MaybeProp::from(button_variant),
    size: MaybeProp::from(button_size),
    on_click: Some(Callback::new(move |_| ...)),
    children: ...
)
```

**Dioxus emitter:**
```rust
dioxus_shadcn_button::Button {
    variant: Some(button_variant),
    size: Some(button_size),
    onclick: Some(EventHandler::new(move |_| ...)),
    children: ...
}
```

Because prop names and event names are identical, the emitter logic is unified.

## 8. Architecture Decision Records (ADRs)

### ADR‑001: JSON Schema for Common Spec
**Decision:** Use JSON Schema to define component APIs.  
**Rationale:** Machine‑readable, supports validation and documentation generation.

### ADR‑002: WebDriver‑Based Conformance Testing
**Decision:** Use `fantoccini` to test actual DOM output.  
**Rationale:** Framework‑agnostic; validates real browser behavior.

### ADR‑003: Incremental Alignment with Deprecation
**Decision:** Allow breaking changes only with a deprecation cycle.  
**Rationale:** Protects existing users while enabling convergence.

## 9. Traceability

- **ASR‑001** → ADR‑001 (JSON Schema)
- **ASR‑002** → ADR‑002 (WebDriver)
- **ASR‑003** → ADR‑003 (Deprecation cycle)
```

---

## 5. Behavioral Specification & Test Verification Plan (`shadcn-rs-api-alignment-test.md`)

```markdown
# Behavioral Specification & Test Verification Plan

| Field | Value |
|-------|-------|
| Project | Shadcn UI API Alignment for Quoin UCP |
| Document | Behavioral Spec & Test Verification |
| Version | 0.1 (Draft) |
| Date | 2026-04-21 |
| Author | System Architect (assisted by AI) |
| Status | Draft — Pending Review |

## 1. Behavioral Specifications (Specification by Example)

These scenarios define the expected behavior for components. They serve as both acceptance criteria and the basis for conformance tests.

### 1.1 Button Component

**Scenario: Default button renders correctly**
- **Given** a Button with variant “default” and children “Click me”
- **When** rendered
- **Then** DOM contains `<button>` with class `bg-primary text-primary-foreground` and text “Click me”
- **And** `role="button"`

**Scenario: Button click fires callback**
- **Given** a Button with `onClick` handler
- **When** clicked
- **Then** handler invoked exactly once.

**Scenario: Disabled button does not fire callback**
- **Given** a Button with `disabled=true` and `onClick` handler
- **When** clicked
- **Then** handler not invoked.

**Scenario: Loading button shows spinner**
- **Given** a Button with `loading=true`
- **When** rendered
- **Then** button contains an SVG with class `animate-spin`
- **And** `aria-busy="true"`.

### 1.2 Input Component

**Scenario: Required field shows error on blur**
- **Given** an Input with `required=true` and empty value
- **When** blurred
- **Then** error message “This field is required” displayed
- **And** `aria-invalid="true"`.

**Scenario: Email validation fails**
- **Given** an Input with `type="email"`
- **When** value “not-an-email” is entered and blurred
- **Then** error message indicates invalid email.

## 2. Test Strategy & Plan

### 2.1 Test Suite Structure

The conformance test suite is a separate crate (`shadcn-rs-conformance`) that both libraries use as a dev‑dependency.

**Running tests for Leptos:**
```bash
cd leptos-shadcn-ui
cargo test --features conformance
```

**Running tests for Dioxus:**
```bash
cd dioxus-shadcn-ui
cargo test --features conformance
```

### 2.2 Test Harness

Each library provides a minimal test harness that:
- Mounts a component with given props.
- Provides a way to simulate events.
- Exposes the DOM for querying.

The conformance suite calls into this harness via a trait.

### 2.3 Coverage Goals

- **Phase 1:** Core components (Button, Input, Card, Dialog, etc.) – 100% scenario coverage.
- **Phase 2:** All remaining components – 100% scenario coverage.

## 3. Test Case Specifications (Examples)

| ID | Requirement | Preconditions | Steps | Expected Result |
|----|-------------|---------------|-------|-----------------|
| TC‑BTN‑001 | F‑BEH‑001 | Button variant=destructive | Render | `<button>` has class `bg-destructive` |
| TC‑INP‑001 | F‑BEH‑003 | Input required=true, empty | Focus then blur | Error message displayed |
| TC‑DLG‑001 | F‑BEH‑002 | Dialog open | Press Tab 5 times | Focus trapped in dialog |

## 4. Conformance Dashboard

A static site (GitHub Pages) displays, for each component:
- Spec version.
- Leptos conformance status (pass/fail).
- Dioxus conformance status (pass/fail).

This provides transparency and motivates alignment.

## 5. Requirements Traceability Matrix (RTM) Excerpt

| SRS Requirement | Test Case | Verification Method |
|-----------------|-----------|---------------------|
| F‑BEH‑001 | TC‑BTN‑001 | Conformance Test |
| F‑BEH‑003 | TC‑INP‑001 | Conformance Test |
| F‑BEH‑002 | TC‑DLG‑001 | Conformance Test |

## 6. Living Documentation

- The common spec is the source of truth for component APIs.
- The conformance dashboard shows real‑time compliance.
- Quoin’s UCP renderer uses the spec to generate code, ensuring it stays in sync.
