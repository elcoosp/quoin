# quoin

[![Crates.io](https://img.shields.io/crates/v/quoin.svg)](https://crates.io/crates/quoin)
[![Docs.rs](https://docs.rs/quoin/badge.svg)](https://docs.rs/quoin)
[![License: MIT](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**One reactive core, many frameworks.**

`quoin` provides a foundational abstraction layer for building framework‑agnostic
reactive libraries in Rust. Write your reactive logic once, and support GPUI,
Dioxus, Leptos, Xilem, Floem, and more—with only a feature flag toggle.

## ✨ Features

- **Reactive Primitives** – `ReactiveContext`, `Signal`, and `Executor` traits.
- **Five Framework Adapters** – GPUI, Leptos, Dioxus, Floem, and Xilem.
- **Declarative UI Macros** – `component!`, `quoin_render!`, and `effect!`.
- **Universal Component Protocol** – Framework‑agnostic Button, Input, Table, VirtualList, Dropdown, and more.
- **Conformance Test Suite** – Guarantees identical behavior across adapters.

## 📦 Usage

Add `quoin` to your `Cargo.toml` and enable **exactly one** adapter feature:

```toml
[dependencies]
quoin = { version = "0.1", features = ["gpui"] }   # or "leptos", "dioxus", "floem", "xilem"
```

Define framework‑agnostic hooks using the core traits:

```rust
use quoin::{ReactiveContext, Signal};

pub fn use_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
    cx.create_signal(0)
}
```

Downstream users select the framework they need, and the appropriate adapter is
used automatically.

## 🧩 Core Abstractions

| Trait | Purpose |
|-------|---------|
| **`ReactiveContext`** | Creates signals and provides the async executor. |
| **`Signal<T>`** | Readable and writable reactive value. |
| **`Executor`** | Spawns futures on the framework's native runtime. |
| **`CancellationToken`** | Cooperative cancellation for async tasks. |

## 🎨 Declarative Macros

`quoin-macros` provides a complete toolkit for writing UI components that work
across all supported frameworks.

### `component!`

Define stateful components with actions and a render block:

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

- **GPUI** → builder chains (`div().flex().flex_col().gap_4()`)
- **Leptos** → `view! { <div class="flex flex-col gap-4">...</div> }`
- **Dioxus** → `rsx! { div { class: "flex flex-col gap-4", ... } }`

```rust
quoin_render! {
    div(class: "flex flex-col gap-4 p-4") {
        h1(class: "text-2xl font-bold") { "Welcome" }
        @for person in people.iter() {
            div(class: "p-2 bg-gray-800 rounded") {
                format!("{} - {}", person.name, person.age)
            }
        }
    }
}
```

### `effect!`

Run side effects that automatically track signal dependencies:

```rust
effect! {
    watch: [count, selected],
    || println!("Count: {}, Selected: {}", count.get(), selected.get())
}
```

## 🔌 Framework Adapters

All adapters are tested against the same conformance suite to guarantee identical
behavior.

| Adapter | Crate | Framework |
|---------|-------|-----------|
| GPUI | `quoin-gpui` | [Zed's GPUI](https://github.com/zed-industries/zed) |
| Leptos | `quoin-leptos` | [Leptos 0.8](https://leptos.dev) |
| Dioxus | `quoin-dioxus` | [Dioxus 0.7](https://dioxuslabs.com) |
| Floem | `quoin-floem` | [Floem](https://github.com/lapce/floem) |
| Xilem | `quoin-xilem` | [Xilem](https://github.com/linebender/xilem) |

## 🧱 Universal Component Protocol (UCP)

`quoin-ui` defines framework‑agnostic adapter traits for complex UI components.
Each framework backend provides native implementations.

| Component | Status |
|-----------|--------|
| Button | ✅ |
| TextInput | ✅ |
| DataTable | ✅ |
| VirtualList | ✅ |
| DropdownMenu | ✅ |
| TabBar | 🚧 |
| RichText | 🚧 |

## 📂 Examples

The repository includes runnable examples for every framework.

### Counter (Core Abstraction)

```bash
# GPUI
cargo run -p counter-gpui

# Leptos
cd examples/counter-leptos && trunk serve

# Dioxus
cargo run -p counter-dioxus
```

### UCP Demo (Full Macro System)

The `ucp-lib` crate demonstrates a complete cross‑framework component library:

```bash
# GPUI native app
cargo run -p ucp-demo-gpui

# Leptos web app
cd examples/ucp-demo-leptos && trunk serve

# Dioxus desktop app
cargo run -p ucp-demo-dioxus
```

All three demos use the **exact same** `ucp-lib` source code.

## 🤝 Contributing

Contributions are welcome! See the [`docs/`](docs/) directory for the complete
specification suite, or open an issue to discuss new adapters or features.

## 📄 License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
