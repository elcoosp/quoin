Here's an enhanced version of your `README.md` with improved structure, additional sections, and more detailed explanations.

```markdown
# Quoin

**One Reactive Core, Many Frameworks**

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

Quoin is a framework-agnostic reactive abstraction layer for Rust. Write your reactive business logic once using signals, effects, and async tasks—then run it with **GPUI**, **Dioxus**, **Leptos**, **Xilem**, or **Floem**.

## Table of Contents

- [Why Quoin?](#why-quoin)
- [Features](#features)
- [Supported Frameworks](#supported-frameworks)
- [Quick Start](#quick-start)
- [Declarative Macros](#declarative-macros)
  - [`component!`](#component)
  - [`quoin_render!`](#quoin_render)
  - [`effect!`](#effect)
  - [`run_app!`](#run_app)
- [Core Traits](#core-traits)
- [Universal Component Protocol (UCP)](#universal-component-protocol-ucp)
- [Crate Overview](#crate-overview)
- [Examples](#examples)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## Why Quoin?

Building cross‑platform Rust UI applications often means choosing a framework and being locked into its reactivity model. Quoin decouples your business logic from the view layer, allowing you to:

- **Write once, run anywhere** – The same reactive hooks work in GPUI, Leptos, Dioxus, and more.
- **Test in isolation** – Mock a `ReactiveContext` and test your logic without spinning up a UI.
- **Switch frameworks easily** – Change a feature flag and recompile—your hooks stay unchanged.
- **Share libraries** – Publish framework‑agnostic crates that work with any Quoin‑supported UI.

## Features

- **Unified Reactive API** — `Signal<T>`, `ReactiveContext`, and `Executor` traits that abstract over framework‑specific implementations.
- **Declarative Macros** — `component!`, `quoin_render!`, `effect!`, and `run_app!` for cross‑framework UI definitions.
- **Tailwind Class Transpilation** — Write Tailwind utility classes in `quoin_render!` and they are automatically converted to GPUI method chains.
- **Universal Component Protocol (UCP)** — Abstract traits for data tables, virtual lists, dropdowns, clipboard, navigation, and more via `quoin-ui`.
- **Cooperative Cancellation** — `CancellationToken` for gracefully stopping long‑running async tasks.
- **Global State** — Provide and consume application‑wide reactive values with `provide_global`/`use_global`.
- **Conformance Testing** — A shared test suite ensures all framework adapters behave identically.

## Supported Frameworks

| Framework | Adapter Crate | Status |
|-----------|---------------|--------|
| [GPUI](https://github.com/zed-industries/zed) | `quoin-gpui` | ✅ Full support |
| [Leptos](https://leptos.dev/) | `quoin-leptos` | ✅ Full support |
| [Dioxus](https://dioxuslabs.com/) | `quoin-dioxus` | ✅ Full support |
| [Xilem](https://github.com/linebender/xilem) | `quoin-xilem` | ✅ Full support |
| [Floem](https://github.com/lapce/floem) | `quoin-floem` | ✅ Full support |

> **Important**: You must enable **exactly one** framework feature in your `Cargo.toml`. Quoin uses compile‑time checks to enforce this.

## Quick Start

Add `quoin` to your `Cargo.toml` with the feature flag for your chosen framework:

```toml
[dependencies]
quoin = { path = "quoin", features = ["gpui"] }  # or "leptos", "dioxus", etc.
```

### Framework-Agnostic Counter Hook

Create a library crate that defines a reusable counter hook:

```rust
// counter-lib/src/lib.rs
use quoin_core::prelude::*;
use std::rc::Rc;

pub struct Counter<S: Signal<u32>> {
    pub count: S,
    pub increment: Rc<dyn Fn()>,
}

pub fn use_counter<C: ReactiveContext>(cx: &C) -> Counter<C::Signal<u32>> {
    let count = cx.create_signal(0u32);
    let increment = {
        let count = count.clone();
        Rc::new(move || count.update(|c| *c += 1))
    };
    Counter { count, increment }
}
```

### Use in GPUI

```rust
// counter-gpui/src/main.rs
use quoin::prelude::*;

struct CounterView {
    counter: counter_lib::Counter<quoin::GpuiSignal<u32>>,
    _ctx: GpuiContext,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = self.counter.count.get();
        div()
            .flex()
            .child(format!("Count: {count}"))
            .child(
                div()
                    .child("Increment")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        (this.counter.increment)();
                    }))
            )
    }
}

run_app!(CounterView);
```

### Use in Leptos

```rust
// counter-leptos/src/lib.rs
use quoin::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let ctx = LeptosContext::new();
    let counter = use_counter(&ctx);

    view! {
        <div>
            <p>"Count: " {move || counter.count.get()}</p>
            <button on:click=move |_| (counter.increment)()>"Increment"</button>
        </div>
    }
}
```

## Declarative Macros

Quoin provides powerful macros that generate framework‑specific code from a single, familiar syntax.

### `component!`

Define a reactive component with state, props, actions, and lifecycle hooks.

```rust
component! {
    pub DemoApp {
        props {
            initial_count: u32 = 0,
        }

        state {
            count: u32 = initial_count,
            selected: String = "Option A".to_string(),
        }

        globals {
            theme: Theme,           // automatically retrieved from context
            router: RouterState observe,  // observe = track changes
        }

        on_mount {
            log::info!("Component mounted");
        }

        on_unmount {
            log::info!("Component unmounted");
        }

        fn increment() {
            count.update(|c| *c += 1);
        }

        fn select_option(opt: String) {
            selected.set(opt);
        }

        render {
            let count_text = format!("Count: {}", count.get());
            
            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div { count_text }
                    button(class: "px-4 py-2 bg-blue-600 rounded-md",
                        on_click: move |_| increment()) {
                        "Increment"
                    }
                }
            }
        }
    }
}
```

**Framework Output:**
- **GPUI**: Generates a `struct DemoApp` with a `Render` impl.
- **Leptos**: Generates a `#[component] fn DemoApp() -> impl IntoView`.
- **Dioxus**: Generates a `#[component] fn DemoApp() -> Element`.

### `quoin_render!`

A JSX‑like declarative UI syntax with Tailwind class transpilation.

```rust
quoin_render! {
    div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
        div(class: "text-2xl font-bold hover:text-blue-400") { "Hello" }
        
        // Conditional rendering
        if[show_details] {
            div(class: "p-2 bg-gray-800 rounded-md") { details }
        }

        // Loops
        for[item in items] {
            div(class: "text-sm") { item.name.clone() }
        }

        // Special components
        tabs(active: active_tab, on_click: move |i| set_active(i)) {
            tab(index: 0, label: "Tab 1")
            tab(index: 1, label: "Tab 2")
        }

        data_table(rows: people, striped: true) {
            column(key: "name", label: "Name", render: |p: &Person| p.name.clone())
            column(key: "age",  label: "Age",  render: |p: &Person| p.age.to_string())
        }

        clipboard_button(copy_text: "Copied!") { "Copy" }
    }
}
```

**Tailwind → GPUI Transpilation**

Classes like `flex`, `p-4`, `bg-gray-900`, `hover:text-blue-400` are automatically converted to GPUI method chains:
```rust
.flex().flex_col().gap(px(16.0)).p(px(16.0)).bg(rgb(0x111827))
.hover(|s| s.text_color(rgb(0x60a5fa)))
```

### `effect!`

Run side effects reactively, with optional cleanup.

```rust
// Legacy syntax
effect! { watch: [count], || println!("Count changed: {}", count.get()) }

// Structured syntax
effect! {
    deps: [query],
    run: || {
        let data = fetch_data(&query.get());
        results.set(data);
    },
    cleanup: || {
        cancel_pending_request();
    }
}
```

**Framework Expansions:**
- **Leptos**: `create_effect` + `on_cleanup`
- **Dioxus**: `use_effect` + `use_drop`
- **GPUI**: The `run` closure is executed, and cleanup is called on drop.

### `run_app!`

Bootstrap your application with a single line.

```rust
run_app!(MyComponent);                         // GPUI with default window options
run_app!(MyComponent, window_opts: custom_opts); // GPUI custom window
run_app!(MyComponent);                         // Leptos/Dioxus mounts to body
```

## Core Traits

### `Signal<T>`

A readable and writable reactive value.

```rust
let signal = cx.create_signal(42u32);
assert_eq!(signal.get(), 42);

signal.with(|v| println!("{v}"));  // Borrow without cloning
signal.set(100);
signal.update(|v| *v += 1);
```

### `ReactiveContext`

The entry point for creating signals and accessing framework services.

```rust
let signal = cx.create_signal(0);               // Create reactive state
let executor = cx.executor();                   // Async task spawner
cx.request_update();                            // Mark UI as dirty

// Global state
cx.provide_global(Theme::Dark);
if let Some(theme) = cx.use_global::<Theme>() {
    // ...
}
```

### `Executor`

Spawn asynchronous tasks that run on the framework's native runtime.

```rust
let handle = executor.spawn(async {
    let data = fetch_from_api().await;
    signal.set(data);
});

// Await the result
let result = handle.await?;

// Or abort early
handle.abort();
```

### `CancellationToken`

Cooperatively cancel long‑running async operations.

```rust
let token = CancellationToken::new();

executor.spawn({
    let token = token.clone();
    async move {
        while !token.is_cancelled() {
            do_work().await;
        }
        cleanup().await;
    }
});

// Later, from another task or UI event
token.cancel();
```

## Universal Component Protocol (UCP)

The `quoin-ui` crate defines abstract traits for complex, framework‑independent UI components. Adapter crates like `quoin-ui-gpui` provide concrete implementations.

Available traits:

| Trait | Purpose |
|-------|---------|
| `VirtualListAdapter` | Efficiently render large, scrollable lists |
| `TableAdapter` | Sortable, striped data tables |
| `TextInputAdapter` | Two‑way bound text inputs |
| `ButtonAdapter` | Styled buttons (primary, ghost, destructive) |
| `DropdownMenuAdapter` | Dropdown menus with items |
| `TabBarAdapter` | Tab navigation |
| `Clipboard` | Read/write system clipboard |
| `Navigator` | Browser‑style routing and history |
| `QuoinTheme` | Theme token resolution |

Example: Using a UCP button

```rust
use quoin_ui::{ButtonAdapter, ButtonVariant};

let adapter = GpuiButtonAdapter::default();
let button = adapter.render(Some("Click Me".into()), ButtonVariant {
    primary: true,
    size: ComponentSize::Medium,
    ..Default::default()
});
```

## Crate Overview

```
quoin/                       # Facade — re‑exports everything
├── quoin-core/              # Core traits (Signal, ReactiveContext, Executor)
├── quoin-macros/            # Proc macro entry points
├── quoin-macros-core/       # Macro parsing and code generation
├── quoin-gpui/              # GPUI adapter
├── quoin-leptos/            # Leptos adapter
├── quoin-dioxus/            # Dioxus adapter
├── quoin-xilem/             # Xilem adapter
├── quoin-floem/             # Floem adapter
├── quoin-ui/                # UCP traits
├── quoin-ui-gpui/           # GPUI UCP implementation
└── quoin-conformance/       # Shared conformance test suite
```

## Examples

The `examples/` directory contains runnable demos for each framework:

| Example | Description | Command |
|---------|-------------|---------|
| `counter-gpui` | Simple counter with GPUI | `just run-gpui` |
| `counter-leptos` | SSR counter with Leptos | `just run-leptos` |
| `counter-dioxus` | Desktop counter with Dioxus | `just run-dioxus` |
| `counter-floem` | Counter with Floem | `just run-floem` |
| `counter-xilem` | Counter with Xilem | `just run-xilem` |
| `ucp-demo-gpui` | UCP components in GPUI | `just run-ucp-gpui` |
| `ucp-demo-dioxus` | UCP components in Dioxus | `just run-ucp-dioxus` |
| `mini-devtools-gpui` | Debug panel with tabs, tables, virtual lists | `just run-mini-devtools` |

## Development

### Prerequisites

- Rust stable (1.80+)
- [`cargo-nextest`](https://nexte.st/) for faster testing
- [`just`](https://github.com/casey/just) for task automation
- [`cargo-watch`](https://crates.io/crates/cargo-watch) for hot‑reload (optional)

### Build & Test

```bash
just build               # Build the entire workspace
just test                # Run all tests (nextest)
just test-quoin          # Test core crates only
just test-quoin-macros   # Test macro parsing
just check               # Format check + clippy
just fix                 # Auto‑fix clippy warnings
```

### Framework‑Specific Tests

```bash
just test-quoin-gpui         # GPUI adapter + conformance
just test-quoin-leptos       # Leptos adapter + conformance
just test-quoin-dioxus       # Dioxus adapter + conformance
just test-quoin-floem        # Floem adapter + conformance
just test-quoin-xilem        # Xilem adapter + conformance

just test-conformance-gpui   # GPUI conformance only
just test-conformance-leptos
just test-conformance-dioxus
just test-conformance-floem
just test-conformance-xilem

just test-macros-ui-all      # All macro UI tests (trybuild)
```

### Watch Mode

```bash
just watch-gpui      # Auto‑rebuild GPUI counter on changes
just watch-dioxus
just watch-floem
just watch-xilem
just watch-ucp-gpui
```

### Macro Expansion

To inspect the generated code for a particular component:

```bash
just expand component="ucp-lib" feature="gpui"
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Setting up the development environment
- Adding a new framework adapter
- Extending the macro syntax
- Running and writing tests

Before submitting a pull request, ensure:

- All tests pass: `just test`
- Code is formatted: `just fmt`
- No clippy warnings: `just lint`

## License

Quoin is licensed under the terms of the MIT License.

See [LICENSE](LICENSE) for details.
