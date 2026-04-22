# Quoin

**One Reactive Core, Many Frameworks**

Quoin is a framework-agnostic reactive abstraction layer for Rust. Write your reactive business logic once using signals, effects, and async tasks—then run it with GPUI, Dioxus, Leptos, Xilem, or Floem.

## Features

- **Unified Reactive API** — `Signal<T>`, `ReactiveContext`, and `Executor` traits that abstract over framework-specific implementations
- **Declarative Macros** — `component!`, `quoin_render!`, `effect!`, and `run_app!` for cross-framework UI definitions
- **Tailwind Transpilation** — Write Tailwind classes in `quoin_render!` and get native GPUI method chains
- **Universal Component Protocol** — Abstract traits for data tables, virtual lists, dropdowns, and more via `quoin-ui`
- **Cooperative Cancellation** — `CancellationToken` for long-running async tasks
- **Conformance Testing** — Shared test suite ensuring all adapters behave identically

## Supported Frameworks

| Framework | Adapter Crate | Status |
|-----------|---------------|--------|
| [GPUI](https://github.com/zed-industries/zed) | `quoin-gpui` | ✅ Full support |
| [Leptos](https://leptos.dev/) | `quoin-leptos` | ✅ Full support |
| [Dioxus](https://dioxuslabs.com/) | `quoin-dioxus` | ✅ Full support |
| [Xilem](https://github.com/linebender/xilem) | `quoin-xilem` | ✅ Full support |
| [Floem](https://github.com/lapce/floem) | `quoin-floem` | ✅ Full support |

## Quick Start

Add `quoin` to your `Cargo.toml` with exactly one framework feature:

```toml
[dependencies]
quoin = { path = "quoin", features = ["gpui"] }
```

### Framework-Agnostic Counter Hook

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
        div().flex().child(format!("Count: {count}"))
            .child(/* button calling self.counter.increment */)
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

### `component!` — Define a component once, render anywhere

```rust
component! {
    pub DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
        }

        render {
            let count_text = format!("Count: {}", count.get());
            
            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    button(class: "px-4 py-2 bg-blue-600 text-white rounded-md",
                        on_click: move |_| count.clone().update(|c| *c += 1)) {
                        "Increment"
                    }
                }
            }
        }
    }
}
```

### `quoin_render!` — Tailwind-like declarative UI

```rust
quoin_render! {
    div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
        div(class: "text-2xl font-bold") { "Hello" }
        button(class: "px-4 py-2 bg-blue-600 rounded-md cursor-pointer",
               on_click: move |_| handle_click()) {
            "Click me"
        }
        if[show_details] {
            div(class: "p-2 bg-gray-800 rounded-md") { details }
        }
        for[item in items] {
            div(class: "text-sm") { item.name.clone() }
        }
    }
}
```

### `effect!` — Reactive side effects

```rust
effect! { deps: [count], run: || println!("Count: {}", count.get()) }
effect! { deps: [query], run: || fetch_data(), cleanup: || cancel_request() }
```

### `run_app!` — One-line app bootstrap

```rust
run_app!(MyComponent);  // GPUI: opens window, sets up reactive context
run_app!(MyComponent);  // Leptos: mounts to body
run_app!(MyComponent);  // Dioxus: launches desktop app
```

## Core Traits

### `Signal<T>` — Reactive value

```rust
let signal = cx.create_signal(42);
signal.get();                    // → 42
signal.with(|v| println!("{}", v));  // borrow without clone
signal.set(100);                  // mutate
signal.update(|v| *v += 1);      // mutate in-place
```

### `ReactiveContext` — Framework entry point

```rust
let signal = cx.create_signal(initial_value);
let executor = cx.executor();
cx.request_update();              // mark UI dirty
cx.provide_global(theme);         // provide to children
let theme = cx.use_global::<Theme>();  // retrieve from parent
```

### `Executor` — Async task spawning

```rust
let handle = executor.spawn(async {
    let data = fetch().await;
    signal.set(data);
});
handle.abort();  // cancel
```

### `CancellationToken` — Cooperative cancellation

```rust
let token = CancellationToken::new();
executor.spawn({
    let token = token.clone();
    async move {
        loop {
            if token.is_cancelled() { break; }
            do_work().await;
        }
    }
});
token.cancel();  // signal all waiters
```

## UCP (Universal Component Protocol)

The `quoin-ui` crate provides abstract traits for complex UI components:

```rust
use quoin_ui::{
    VirtualListAdapter, TableAdapter, TextInputAdapter,
    ButtonAdapter, DropdownMenuAdapter, TabBarAdapter,
    Clipboard, Navigator, SortDirection,
};
```

Framework-specific implementations live in adapter crates (e.g., `quoin-ui-gpui`).

## Crate Overview

```
quoin/                    # Facade — re-exports everything
├── quoin-core/           # Core traits (Signal, ReactiveContext, Executor)
├── quoin-macros/         # Proc macro entry points
├── quoin-macros-core/    # Macro parsing and code generation
├── quoin-gpui/           # GPUI adapter
├── quoin-leptos/         # Leptos adapter
├── quoin-dioxus/         # Dioxus adapter
├── quoin-xilem/          # Xilem adapter
├── quoin-floem/          # Floem adapter
├── quoin-ui/             # UCP traits
├── quoin-ui-gpui/        # GPUI UCP implementation
└── quoin-conformance/    # Shared conformance test suite
```

## Development

### Prerequisites

- Rust nightly (for some proc-macro features)
- `cargo-nextest` for testing
- `cargo-watch` for dev workflows
- `just` for task running

### Build & Test

```bash
just build              # Build workspace
just test               # Run all tests (nextest)
just test-quoin         # Test core only
just test-quoin-macros  # Test macro parsing
just check              # Format check + clippy
just fix                # Auto-fix clippy warnings
```

### Framework-Specific Tests

```bash
just test-quoin-gpui        # GPUI adapter + conformance
just test-quoin-leptos      # Leptos adapter + conformance
just test-quoin-dioxus      # Dioxus adapter + conformance
just test-quoin-floem       # Floem adapter + conformance
just test-quoin-xilem       # Xilem adapter + conformance
just test-conformance-gpui  # GPUI conformance only
just test-conformance-leptos
just test-conformance-dioxus
just test-macros-ui-all     # All macro UI tests (trybuild)
```

### Run Examples

```bash
just run-gpui           # GPUI counter
just run-dioxus         # Dioxus counter
just run-leptos         # Leptos counter (SSR)
just run-floem          # Floem counter
just run-xilem          # Xilem counter
just run-ucp-gpui       # UCP demo (GPUI)
just run-ucp-dioxus     # UCP demo (Dioxus)
just run-mini-devtools  # Mini devtools (GPUI)
```

### Watch Mode

```bash
just watch-gpui         # Auto-rebuild GPUI counter
just watch-dioxus       # Auto-rebuild Dioxus counter
just watch-floem        # Auto-rebuild Floem counter
just watch-xilem        # Auto-rebuild Xilem counter
just watch-ucp-gpui     # Auto-rebuild UCP demo
just watch-mini-devtools
```

### Build All Examples

```bash
just build-examples     # Compile-check every example
```

### Macro Expansion

```bash
just expand ucp-lib gpui   # Expand macros to examples/ucp-lib/expanded.rs
```

### Cleanup

```bash
just clean            # cargo clean
just clean-trybuild   # Clear trybuild cache
```

## License

MIT OR Apache-2.0
