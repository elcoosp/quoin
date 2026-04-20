# quoin

> *"One reactive core, many frameworks."*

**`quoin`** is the foundational reactive abstraction layer for the Rust UI ecosystem. It enables library authors to write reactive logic **once** and support multiple UI frameworks—GPUI, Dioxus, Leptos, Xilem, Floem, and beyond—with only a feature flag toggle.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](#)
[![Status: Alpha](https://img.shields.io/badge/status-alpha-yellow.svg)](#)

---

## 📖 Table of Contents

- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [Current Status](#-current-status)
- [Architecture at a Glance](#-architecture-at-a-glance)
- [Getting Started](#-getting-started)
- [Core Abstractions](#-core-abstractions)
- [Framework Adapters](#-framework-adapters)
- [Declarative UI Macros](#-declarative-ui-macros)
- [Universal Component Protocol (UCP)](#-universal-component-protocol-ucp)
- [Examples](#-examples)
- [Specification Suite](#-specification-suite)
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

## 🚦 Current Status

`quoin` is **actively developed** and **production‑ready for early adopters**. All major components are implemented and tested.

| Component | Status | Description |
|-----------|--------|-------------|
| **`quoin` core** | ✅ Stable | `ReactiveContext`, `Signal`, `Executor`, `CancellationToken` |
| **`quoin-gpui` adapter** | ✅ Stable | Full GPUI integration, including view update notifier |
| **`quoin-leptos` adapter** | ✅ Stable | Leptos 0.8 integration |
| **`quoin-dioxus` adapter** | ✅ Stable | Dioxus 0.7 integration |
| **`quoin-floem` adapter** | ✅ Stable | Floem integration |
| **`quoin-xilem` adapter** | ✅ Stable | Xilem integration |
| **`quoin-macros`** | ✅ Stable | `component!`, `quoin_render!`, `quoin_element!` |
| **`quoin-ui`** | ✅ Stable | Universal Component Protocol traits |
| **`quoin-ui-gpui`** | 🚧 In progress | Native GPUI implementations of UCP components |
| **Conformance Suite** | ✅ Complete | All adapters pass the full test suite |

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
```

All adapters are **equal**. Any crate that implements `ReactiveContext` and passes the conformance test suite is a first‑class `quoin` adapter.

---

## 🚀 Getting Started

### Add `quoin` to Your Library

```toml
[dependencies]
quoin = "0.1"
```

### Select an Adapter in Your Application

```toml
# For GPUI (Zed's native GUI framework)
quoin = { version = "0.1", features = ["gpui"] }

# For Leptos (web)
quoin = { version = "0.1", features = ["leptos"] }

# For Dioxus (web & desktop)
quoin = { version = "0.1", features = ["dioxus"] }
```

### Write Framework‑Agnostic Code

```rust
use quoin::{ReactiveContext, Signal};

pub struct Counter<S: Signal<u32>> {
    pub count: S,
    pub increment: std::rc::Rc<dyn Fn()>,
}

pub fn use_counter<C: ReactiveContext>(cx: &C) -> Counter<C::Signal<u32>> {
    let count = cx.create_signal(0u32);
    let increment = {
        let count = count.clone();
        std::rc::Rc::new(move || count.update(|c| *c += 1))
    };
    Counter { count, increment }
}
```

See the [`examples/`](examples/) directory for complete, runnable examples.

---

## 🧩 Core Abstractions

### `ReactiveContext`

The entry point for creating signals and accessing the async executor.

```rust
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    type Signal<T: Clone + 'static>: Signal<T>;
    type Executor: Executor;

    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T>;
    fn executor(&self) -> Self::Executor;
    fn request_update(&self);
}
```

### `Signal<T>`

A readable and writable reactive value.

```rust
pub trait Signal<T: Clone + 'static>: Clone {
    fn get(&self) -> T;
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U;
    fn set(&self, value: T);
    fn update(&self, f: impl FnOnce(&mut T));
}
```

### `Executor`

Abstraction over framework‑specific async runtimes.

```rust
pub trait Executor: Clone + Send + Sync + 'static {
    type JoinHandle<T: Send + 'static>: JoinHandle<T>;
    fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
```

---

## 🔌 Framework Adapters

`quoin` ships with reference adapters for five major frameworks. All adapters are **tested against the same conformance suite** to guarantee identical behavior.

| Adapter | Crate | Framework | Key Integration |
|---------|-------|-----------|-----------------|
| **GPUI** | `quoin-gpui` | [Zed's GPUI](https://github.com/zed-industries/zed) | Manual view invalidation via `set_view_update_notifier` |
| **Leptos** | `quoin-leptos` | [Leptos 0.8](https://leptos.dev) | `RwSignal` backing; automatic reactivity |
| **Dioxus** | `quoin-dioxus` | [Dioxus 0.7](https://dioxuslabs.com) | `Signal` backing; automatic reactivity |
| **Floem** | `quoin-floem` | [Floem](https://github.com/lapce/floem) | `RwSignal` backing |
| **Xilem** | `quoin-xilem` | [Xilem](https://github.com/linebender/xilem) | Thread‑safe signals with tokio runtime |

---

## 🎨 Declarative UI Macros

`quoin-macros` provides a complete suite of procedural macros for writing framework‑agnostic component code.

### `component!`

Define reactive components with state, actions, and a render block. The same macro expands to GPUI structs, Leptos components, or Dioxus components based on the active feature flag.

```rust
component! {
    pub Counter {
        state {
            count: u32 = 0,
        }

        fn increment(&self) {
            self.count.update(|c| *c += 1);
        }

        render {
            quoin_render! {
                div(class: "flex gap-2") {
                    div { format!("Count: {}", self.count.get()) }
                    button(on_click: |this: &mut Counter| this.increment()) {
                        "Increment"
                    }
                }
            }
        }
    }
}
```

### `quoin_render!`

Declarative, Tailwind‑inspired view syntax that transpiles to native framework code:

- **GPUI:** Expands to builder chains (`div().flex().flex_col().gap_4()`)
- **Leptos:** Expands to `view! { <div class="flex flex-col gap-4">...</div> }`
- **Dioxus:** Expands to `rsx! { div { class: "flex flex-col gap-4", ... } }`

```rust
quoin_render! {
    div(class: "flex flex-col gap-4 p-4") {
        h1(class: "text-2xl font-bold") { "Welcome" }
        @for person in people.iter() {
            div(class: "p-2 bg-gray-800 rounded") {
                format!("{} - {}", person.name, person.age)
            }
        }
        @if count.get() > 5 {
            div(class: "text-red-500") { "High count!" }
        }
    }
}
```

### `effect!`

Run side effects that automatically track signal dependencies.

```rust
effect! {
    watch: [count, selected],
    || {
        println!("Count is {}, selected is {}", count.get(), selected.get());
    }
}
```

---

## 🧱 Universal Component Protocol (UCP)

`quoin-ui` defines framework‑agnostic adapter traits for complex UI components. Each framework backend provides native implementations.

| Component | Trait | Status |
|-----------|-------|--------|
| **Button** | `ButtonAdapter` | ✅ Implemented |
| **TextInput** | `TextInputAdapter` | ✅ Implemented |
| **DataTable** | `TableAdapter` | ✅ Implemented |
| **VirtualList** | `VirtualListAdapter` | ✅ Implemented |
| **DropdownMenu** | `DropdownAdapter` | ✅ Implemented |
| **TabBar** | `TabBarAdapter` | 🚧 In progress |
| **RichText** | `RichTextAdapter` | 🚧 In progress |

Theme tokens (`ThemeToken`) provide a unified color system that maps to each framework's theming mechanism.

---

## 📂 Examples

The `examples/` directory contains runnable demonstrations for every framework.

### Counter (Core Abstraction)

| Example | Framework | Run Command |
|---------|-----------|-------------|
| `counter-gpui` | GPUI | `cargo run -p counter-gpui` |
| `counter-leptos` | Leptos | `cd examples/counter-leptos && trunk serve` |
| `counter-dioxus` | Dioxus | `cargo run -p counter-dioxus` |
| `counter-floem` | Floem | `cargo run -p counter-floem` |
| `counter-xilem` | Xilem | `cargo run -p counter-xilem` |

### UCP Demo (Macros + Shared Library)

The `ucp-lib` crate demonstrates a complete cross‑framework component library:

```bash
# GPUI native app
cargo run -p ucp-demo-gpui

# Leptos web app
cd examples/ucp-demo-leptos && trunk serve

# Dioxus desktop app
cargo run -p ucp-demo-dioxus
```

All three demos use the **exact same** `ucp-lib` source code, proving `quoin`'s write‑once, run‑anywhere capability.

---

## 📚 Specification Suite

The `quoin` project is guided by a complete, evidence‑backed specification suite. Each document is traceable to the next, forming a rigorous foundation for implementation.

| Document | Description |
|----------|-------------|
| **[Vision & Strategic Alignment](docs/vision.md)** | The "why" – problem statement, target users, success metrics |
| **[Business & Stakeholder Requirements (BRS)](docs/brs.md)** | Stakeholder goals, business rules, conceptual domain model |
| **[Software Requirements Specification (SRS)](docs/srs.md)** | Functional and non‑functional requirements (EARS‑style) |
| **[Architecture & Design Specification](docs/architecture.md)** | Structural design, ADRs, API contracts |
| **[Behavioral Spec & Test Verification](docs/test-verification.md)** | BDD scenarios, conformance test suite, traceability matrix |

📁 **All documents are available in the [`docs/`](docs/) directory.**

---

## 🤝 Getting Involved

`quoin` is a community‑driven project. We welcome contributions of all kinds!

### For Library Authors
- **Adopt `quoin`:** Make your crate framework‑agnostic.
- **Provide feedback:** Open an issue to discuss your use case or pain points.

### For Framework Enthusiasts
- **Write an adapter:** Implement `ReactiveContext` for your favorite UI framework.
- **Get listed:** Once your adapter passes the conformance suite, submit a PR to have it listed here.

### Development

```bash
# Clone the repository
git clone https://github.com/elcoosp/quoin.git
cd quoin

# Run all tests
cargo nextest run --all

# Run specific adapter tests
just test-quoin-gpui
just test-quoin-leptos

# Run macro UI tests
just test-macros-ui-all
```

See the [`justfile`](justfile) for all available commands.

---

## 📄 License

All `quoin` crates are dual‑licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

---

*Built with ❤️ for the Rust UI ecosystem.*
