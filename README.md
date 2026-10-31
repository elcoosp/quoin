<div align="center">
  <img src="docs/logo.png" alt="Quoin Logo" width="200"/>
  <p>
    <strong>One Reactive Core, Many Frameworks.</strong><br/>
    A framework-agnostic reactive abstraction layer for Rust. Write your reactive business logic once using signals, effects, and async tasks—then run it with GPUI, Dioxus, Leptos, Xilem, or Floem.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.80%2B-orange?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="License"/>
    <img src="https://img.shields.io/badge/Frameworks-GPUI%20%7C%20Leptos%20%7C%20Dioxus%20%7C%20Xilem%20%7C%20Floem-262D3A?style=flat-square" alt="Frameworks"/>
    <img src="https://img.shields.io/badge/Architecture-10%2B%20Crates-6F4E37?style=flat-square" alt="Crates"/>
    <img src="https://img.shields.io/badge/Reactivity-Signals%20%2B%20Effects-6A0DAD?style=flat-square" alt="Reactivity"/>
    <img src="https://img.shields.io/badge/Macros-Tailwind%20Transpilation-007ACC?style=flat-square" alt="Macros"/>
    <img src="https://img.shields.io/badge/Testing-Conformance%20%2B%20Snapshot-00BFFF?style=flat-square" alt="Testing"/>
  </p>
</div>

---

## Overview

Building cross-platform Rust UI applications often means choosing a framework and being locked into its reactivity model. Quoin decouples your business logic from the view layer, allowing you to:

- **Write once, run anywhere**: The same reactive hooks work in GPUI, Leptos, Dioxus, and more.
- **Test in isolation**: Mock a `ReactiveContext` and test your logic without spinning up a UI.
- **Switch frameworks easily**: Change a feature flag and recompile—your hooks stay unchanged.
- **Share libraries**: Publish framework-agnostic crates that work with any Quoin-supported UI.

## Features

- **Unified Reactive API**: `Signal<T>`, `ReactiveContext`, and `Executor` traits abstract over framework-specific implementations.
- **Declarative Macros**: `component!`, `quoin_render!`, `effect!`, and `run_app!` enable cross-framework UI definitions.
- **Tailwind Class Transpilation**: Write Tailwind utility classes in `quoin_render!` and have them automatically converted to framework-specific method chains (e.g., GPUI).
- **Universal Component Protocol (UCP)**: Abstract traits for data tables, virtual lists, dropdowns, clipboard, and more via `quoin-ui`.
- **Cooperative Cancellation**: `CancellationToken` for gracefully stopping long-running async tasks.
- **Global State**: Provide and consume application-wide reactive values with `provide_global` / `use_global`.
- **Conformance Testing**: A shared test suite ensures all framework adapters behave identically.

## Supported Frameworks

| Framework | Adapter Crate | Status |
|-----------|---------------|--------|
| [GPUI](https://github.com/zed-industries/zed) | `quoin-gpui` | ✅ Full support |
| [Leptos](https://leptos.dev/) | `quoin-leptos` | ✅ Full support |
| [Dioxus](https://dioxuslabs.com/) | `quoin-dioxus` | ✅ Full support |
| [Xilem](https://github.com/linebender/xilem) | `quoin-xilem` | ✅ Full support |
| [Floem](https://github.com/lapce/floem) | `quoin-floem` | ✅ Full support |

> [!IMPORTANT]
> You must enable **exactly one** framework feature in your `Cargo.toml`. Quoin uses compile-time checks to enforce this.

## Quick Start

Add `quoin` to your `Cargo.toml` with the feature flag for your chosen framework:

```toml
[dependencies]
quoin = { path = "quoin", features = ["gpui"] }  # or "leptos", "dioxus", etc.
```

### 1. Define Framework-Agnostic Logic

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

### 2. Use in a Framework (e.g., GPUI)

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

## Declarative Macros

Quoin provides powerful macros that generate framework-specific code from a single, familiar syntax.

### `component!`
Define a reactive component with state, props, actions, and lifecycle hooks.

```rust
component! {
    pub DemoApp {
        props { initial_count: u32 = 0 }
        state { count: u32 = initial_count }
        globals { theme: Theme, router: RouterState observe }

        on_mount { log::info!("Component mounted"); }
        on_unmount { log::info!("Component unmounted"); }

        fn increment() { count.update(|c| *c += 1); }

        render {
            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div { format!("Count: {}", count.get()) }
                    button(class: "px-4 py-2 bg-blue-600 rounded-md", on_click: move |_| increment()) {
                        "Increment"
                    }
                }
            }
        }
    }
}
```

### `quoin_render!`
A JSX-like declarative UI syntax with automatic Tailwind class transpilation.

```rust
quoin_render! {
    div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
        div(class: "text-2xl font-bold hover:text-blue-400") { "Hello" }
        
        if[show_details] {
            div(class: "p-2 bg-gray-800 rounded-md") { details }
        }

        for[item in items] {
            div(class: "text-sm") { item.name.clone() }
        }

        data_table(rows: people, striped: true) {
            column(key: "name", label: "Name", render: |p: &Person| p.name.clone())
            column(key: "age",  label: "Age",  render: |p: &Person| p.age.to_string())
        }
    }
}
```

> [!NOTE]
> Classes like `flex`, `p-4`, `bg-gray-900`, and `hover:text-blue-400` are automatically converted to framework-specific method chains (e.g., `.flex().flex_col().p(px(16.0)).bg(rgb(0x111827))` in GPUI).

### `effect!`
Run side effects reactively, with optional cleanup.

```rust
effect! {
    deps: [query],
    run: || {
        let data = fetch_data(&query.get());
        results.set(data);
    },
    cleanup: || { cancel_pending_request(); }
}
```

### `run_app!`
Bootstrap your application with a single line.

```rust
run_app!(MyComponent);                         // GPUI with default window options
run_app!(MyComponent, window_opts: custom_opts); // GPUI custom window
run_app!(MyComponent);                         // Leptos/Dioxus mounts to body
```

## Core Traits

| Trait | Purpose |
|-------|---------|
| `Signal<T>` | A readable and writable reactive value (`get`, `set`, `update`, `with`). |
| `ReactiveContext` | Entry point for creating signals, accessing framework services, and global state (`provide_global`, `use_global`). |
| `Executor` | Spawn asynchronous tasks that run on the framework's native runtime. |
| `CancellationToken` | Cooperatively cancel long-running async operations. |

## Universal Component Protocol (UCP)

The `quoin-ui` crate defines abstract traits for complex, framework-independent UI components. Adapter crates (e.g., `quoin-ui-gpui`) provide concrete implementations.

- **`VirtualListAdapter`**: Efficiently render large, scrollable lists.
- **`TableAdapter`**: Sortable, striped data tables.
- **`TextInputAdapter`**: Two-way bound text inputs.
- **`ButtonAdapter`**: Styled buttons (primary, ghost, destructive).
- **`DropdownMenuAdapter`**: Dropdown menus with items.
- **`TabBarAdapter`**: Tab navigation.
- **`Clipboard`**: Read/write system clipboard.
- **`QuoinTheme`**: Theme token resolution.

## Crate Overview

```text
quoin/                       # Facade — re-exports everything
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

## Development

### Prerequisites
- Rust stable (1.80+)
- [`cargo-nextest`](https://nexte.st/) for faster testing
- [`just`](https://github.com/casey/just) for task automation

### Common Tasks
```bash
just build               # Build the entire workspace
just test                # Run all tests (nextest)
just check               # Format check + clippy
just fix                 # Auto-fix clippy warnings
```

### Framework-Specific Tests
```bash
just test-quoin-gpui         # GPUI adapter + conformance
just test-quoin-leptos       # Leptos adapter + conformance
just test-conformance-gpui   # GPUI conformance only
just test-macros-ui-all      # All macro UI tests (trybuild)
```

### Watch Mode
```bash
just watch-gpui      # Auto-rebuild GPUI counter on changes
just watch-dioxus
```

### Macro Expansion
To inspect the generated code for a particular component:
```bash
just expand component="ucp-lib" feature="gpui"
```

## Examples

The `examples/` directory contains runnable demos for each framework:

| Example | Description | Command |
|---------|-------------|---------|
| `counter-gpui` | Simple counter with GPUI | `just run-gpui` |
| `counter-leptos` | SSR counter with Leptos | `just run-leptos` |
| `counter-dioxus` | Desktop counter with Dioxus | `just run-dioxus` |
| `ucp-demo-gpui` | UCP components in GPUI | `just run-ucp-gpui` |
| `mini-devtools-gpui` | Debug panel with tabs, tables, virtual lists | `just run-mini-devtools` |
