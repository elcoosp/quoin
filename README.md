<div align="center">
  <img src="docs/logo.png" alt="Quoin Logo" width="300"/>
  <p>
    <strong>One Reactive Core, Many Frameworks.</strong> <br/>
    A framework-agnostic reactive abstraction layer for Rust. Write your reactive business logic once using signals, effects, and async tasks—then run it with GPUI, Dioxus, Leptos, Xilem, or Floem.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.88%2B-orange?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License"/>
    <img src="https://img.shields.io/badge/Frameworks-GPUI%20%7C%20Leptos%20%7C%20Dioxus%20%7C%20Xilem%20%7C%20Floem-262D3A?style=flat-square" alt="Frameworks"/>
    <img src="https://img.shields.io/badge/Architecture-10%2B%20Crates-6F4E37?style=flat-square" alt="Crates"/>
    <img src="https://img.shields.io/badge/Reactivity-Signals%20%2B%20Effects-6A0DAD?style=flat-square" alt="Reactivity"/>
    <img src="https://img.shields.io/badge/Macros-Tailwind%20Transpilation-007ACC?style=flat-square" alt="Macros"/>
    <img src="https://img.shields.io/badge/Testing-Conformance%20%2B%20Testkit-00BFFF?style=flat-square" alt="Testing"/>
    <img src="https://img.shields.io/badge/Devtools-Signal%20Tracing-FF6B6B?style=flat-square" alt="Devtools"/>
  </p>
</div>

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

## Why Quoin?
Building cross‑platform Rust UI applications often means choosing a framework and being locked into its reactivity model. Quoin decouples your business logic from the view layer, allowing you to:
- **Write once, run anywhere** – The same reactive hooks work in GPUI, Leptos, Dioxus, and more.
- **Test in isolation** – Mock a `ReactiveContext` and test your logic without spinning up a UI.
- **Switch frameworks easily** – Change a feature flag and recompile—your hooks stay unchanged.
- **Share libraries** – Publish framework‑agnostic crates that work with any Quoin‑supported UI.

## Features
- **Unified Reactive API** — `Signal<T>`, `ReactiveContext`, and `Executor` traits that abstract over framework‑specific implementations.
- **Derived Signals** — `derived(&source, |v| …)`, `.map()`, and `zip()` computed views that work identically on every adapter.
- **Reactive Effects** — `create_effect(run)` and `watch(signal, f)`: push-based, dependency-tracked reactions implemented once in `quoin-core` (batch-aware flush, dynamic dependency sets, `EffectScope` disposal) — no per-adapter effect system required. The `effect!` macro lowers to this runtime on Xilem (component-scoped via `quoin::xilem_effect`) and to the frameworks' own reactive graphs everywhere else — native on all five backends.
- **Resources** — `cx.create_resource(fetch)` gives every framework the Leptos/Dioxus async-data lifecycle (`Loading → Ready | Failed`) with race-free reloads and cooperative cancellation.
- **Batched Updates** — `cx.batch(|| …)` collapses N signal writes into one re-render request on GPUI/Xilem.
- **`spawn_local` Everywhere** — `LocalExecutor` runs `!Send` futures: platform-driven on WASM, deterministic thread-local queue on native.
- **Rich Cancellation** — `CancellationToken` with `child_token()`, `guard()`, `into_cancelled()`, and `run_until_cancelled()`.
- **Declarative Macros** — `component!`, `quoin_render!`, `effect!`, and `run_app!` for cross‑framework UI definitions.
- **Tailwind Class Transpilation** — Write Tailwind utility classes in `quoin_render!` and they are automatically converted to GPUI method chains.
- **Universal Component Protocol (UCP)** — Abstract traits for data tables, virtual lists, dropdowns, clipboard, and more via `quoin-ui`.
- **Global State (Shared)** — `provide_global`/`use_global` with unified *shared* semantics on all five adapters; `update_global` and `use_global_or_init` helpers.
- **Testkit** — `quoin-testkit` runs framework-agnostic logic headlessly with deterministic executors and update-count assertions.
- **Devtools** — `quoin-devtools` records every signal mutation, update request, and task spawn into an inspectable log (`QUOIN_TRACE=1`).
- **Conformance Testing** — A shared suite (30+ checks per adapter) ensures all framework adapters behave identically.

## Supported Frameworks
| Framework | Adapter Crate | Status |
| --- | --- | --- |
| [GPUI](https://github.com/zed-industries/zed) | quoin-gpui | ✅ Full support |
| [Leptos](https://leptos.dev/) | quoin-leptos | ✅ Full support |
| [Dioxus](https://dioxuslabs.com/) | quoin-dioxus | ✅ Full support |
| [Xilem](https://github.com/linebender/xilem) | quoin-xilem | ✅ Full support |
| [Floem](https://github.com/lapce/floem) | quoin-floem | ✅ Full support |

> **Important:** You must enable exactly one framework feature in your `Cargo.toml`. Quoin uses compile‑time checks to enforce this.

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
- **GPUI:** Generates a `struct DemoApp` with a `Render` impl.
- **Leptos:** Generates a `#[component] fn DemoApp() -> impl IntoView`.
- **Dioxus:** Generates a `#[component] fn DemoApp() -> Element`.

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
- **Leptos:** `create_effect` + `on_cleanup`
- **Dioxus:** `use_effect` + `use_drop`
- **GPUI:** The `run` closure is executed, and cleanup is called on drop.
- **Floem:** `floem::reactive::Effect::new`; `cleanup:` runs before each re-run and when the owning reactive scope is disposed (`quoin::floem_effect::CleanupSlot`)
- **Xilem:** the core reactive runtime (`quoin::xilem_effect::create_effect`) observing the component's `XilemSignal`s through their dependency cells; `cleanup:` runs before each re-run and when the effect is disposed with its component scope (`quoin::xilem_effect::CleanupSlot`). Effects belong in `on_mount { .. }` — `effect!` in the render body is a named compile error (a rebuild would re-create the effect every time)

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

// Await the result (Err(Canceled) if the task was aborted)
let result = handle.await?;
```
Every `JoinHandle` implements the same lifecycle semantics on all five adapters:

| Operation | Effect |
| --- | --- |
| `handle.await` | Waits for the result. The task runs to completion. |
| `handle.abort()` | Cancels the task at its next suspension point. Later awaits resolve with `Err(Canceled)`. |
| `handle.detach()` | Fire-and-forget: the task runs to completion, its result is discarded. |
| `drop(handle)` | Cancels the task (structured concurrency — no orphaned tasks mutating UI state). Use `detach()` when you explicitly want fire-and-forget. |

*Cancellation is cooperative on the thread-based adapters (Leptos, Dioxus, Floem): the future stops at its next `.await` point. GPUI and Xilem cancel natively (dropping a GPUI `Task` / aborting a tokio task). On `wasm32` targets, the Leptos and Dioxus executors run futures on the main thread via `wasm_bindgen_futures` instead of spawning threads.*

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

## Beyond the Basics

The full API surface every adapter shares (import with `use quoin_core::prelude::*;` or `use quoin::prelude::*;`):

### Derived signals (computed views)
```rust
let count = cx.create_signal(2u32);
let label = count.map(|n| format!("count = {n}"));   // recomputed on read
assert_eq!(label.get(), "count = 2");
let both = quoin_core::zip(&a, &b);                  // tuple of two signals
```

### Resources (async data loading)
```rust
let user = cx.create_resource(|token| async move {
    // real fetch here; check `token.is_cancelled()` between awaits
    Ok::<User, FetchError>(fetch_user().await?)
});
// UI reads: user.state().get() -> Loading | Ready(User) | Failed(err)
// Reloads cancel the in-flight request and can never be overwritten by it.
```

### Batched updates
```rust
cx.batch(|| {
    a.set(1);
    b.set(2);
});  // GPUI/Xilem: one re-render request instead of two
```

### `spawn_local` for `!Send` futures
```rust
use quoin_core::LocalExecutor;
// wasm: runs on the browser main thread. native: deterministic thread-local
// queue driven by `quoin_core::local::run_local_tasks()`.
cx.executor().spawn_local(async { update_dom().await });
```

### Cancellation toolkit
```rust
let parent = CancellationToken::new();
let per_request = parent.child_token();     // cancels with the parent
let _guard = parent.guard();                // RAII: cancels on drop
let value = parent.run_until_cancelled(work()).await;  // race work vs cancel
```

### Reactive effects (push-based reactivity)
```rust
use quoin_core::EffectScope;

let query = cx.create_signal(String::new());
let results = cx.create_signal(Vec::new());

let scope = EffectScope::new();
let (query2, results2) = (query.clone(), results.clone());
// Re-runs whenever the signals it reads change; batched writes coalesce
// into one re-run; dropping `scope` disposes the effect (component lifetime).
scope.create_effect(move || {
    results2.set(search(&query2.get()));
});

// Or observe a single signal explicitly (fires on writes, not initially):
let _handle = quoin_core::watch(&query, |q| println!("query = {q}"));
```
Effects track their dependencies automatically (the set is rebuilt every
run, so conditional reads produce dynamic graphs), defer self-writes instead
of recursing, and pay one thread-local flag check per read when idle. Type-erase
signal handles for storage with `AnySignal<T>` (ADR-002's opt-in dynamic
dispatch — zero measurable read overhead, see `docs/BENCHMARKS.md`).

### Signal extras
```rust
signal.set_eq(new_value);   // writes only when changed (fewer renders)
let old = signal.replace(v); signal.take();  // Default backfill
```

### Global state helpers
```rust
cx.update_global::<Settings, ()>(|s| s.theme = Dark);
let settings = cx.use_global_or_init(Settings::default);
```

## Universal Component Protocol (UCP)
The `quoin-ui` crate defines abstract traits for complex, framework‑independent UI components. Adapter crates like `quoin-ui-gpui` provide concrete implementations.

**Available traits:**

| Trait | Purpose |
| --- | --- |
| `VirtualListAdapter` | Efficiently render large, scrollable lists |
| `TableAdapter` | Sortable, striped data tables |
| `TextInputAdapter` | Two‑way bound text inputs |
| `ButtonAdapter` | Styled buttons (primary, ghost, destructive) |
| `DropdownMenuAdapter` | Dropdown menus with items |
| `TabBarAdapter` | Tab navigation |
| `Clipboard` | Read/write system clipboard |
| `QuoinTheme` | Theme token resolution |

**Example: Using a UCP button**
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
```text
quoin/                       # Facade — re‑exports everything
├── quoin-core/              # Core traits + derived/resource/batch/local/pool
│                          #   + effect (push-based reactive effects),
│                          #   + AnySignal (type-erased handles, ADR-002),
│                          #   + shared adapter machinery: join (JoinHandle
│                          #   lifecycle), globals (thread-local store),
│                          #   NotifierSignal (cell + batch-aware notifier)
├── quoin-macros/            # Proc macro entry points
├── quoin-macros-core/       # Macro parsing and code generation
├── quoin-gpui/              # GPUI adapter
├── quoin-leptos/            # Leptos adapter
├── quoin-dioxus/            # Dioxus adapter
├── quoin-xilem/             # Xilem adapter
├── quoin-floem/             # Floem adapter
├── quoin-ui/                # UCP traits
├── quoin-ui-gpui/           # GPUI UCP implementation
├── quoin-testkit/           # Headless testing: TestContext + executors
├── quoin-devtools/          # Signal tracing + update logging
└── quoin-conformance/       # Shared conformance test suite (30+ per adapter)
```

## Examples
The `examples/` directory contains runnable demos for each framework:

| Example | Description | Command |
| --- | --- | --- |
| counter-gpui | Simple counter with GPUI | `just run-gpui` |
| counter-leptos | SSR counter with Leptos | `just run-leptos` |
| counter-dioxus | Desktop counter with Dioxus | `just run-dioxus` |
| counter-floem | Counter with Floem | `just run-floem` |
| counter-xilem | Counter with Xilem + **live devtools trace panel** | `cargo run --manifest-path examples/counter-xilem/Cargo.toml` |
| ucp-demo-gpui | UCP components in GPUI | `just run-ucp-gpui` |
| ucp-demo-dioxus | UCP components in Dioxus | `just run-ucp-dioxus` |
| mini-devtools-gpui | Debug panel with tabs, tables, virtual lists | `just run-mini-devtools` |
| **todo-core** | Framework-agnostic todo store, e2e tested | `cargo test -p todo-core` |
| **fetch-core** | Resource-driven async search, e2e tested | `cargo test -p fetch-core` |
| **devtools-demo** | Headless signal-tracing session | `just trace-demo` |

## Development

### Prerequisites
- Rust stable (1.88+; declared as `rust-version = "1.88"` in the workspace)
- [`cargo-nextest`](https://nexte.st/) for faster testing
- [`just`](https://github.com/casey/just) for task automation
- [`cargo-watch`](https://crates.io/crates/cargo-watch) for hot‑reload (optional)

### Build & Test
```bash
just build               # Build the entire workspace
just test                # Run all tests (nextest)
just test-quoin          # Test the facade crate
just test-quoin-macros   # Test macro parsing
just check               # Format check + clippy (framework-free crates)
just check-all           # Format check + clippy (everything, incl. GPUI)
just fix                 # Auto‑fix clippy warnings
just bench-core          # Micro-benchmarks for the core primitives (release)
just trace-demo          # Headless devtools session with QUOIN_TRACE=1
```
*`just check` only lints the framework-free crates, so it runs without any GPUI system libraries. `just check-all` additionally lints the GPUI crates and needs `libxkbcommon-dev`, `libx11-dev`, `libxcb1-dev`, `libwayland-dev` and `libfontconfig1-dev` on Linux.*

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
just test-testkit            # quoin-testkit suite
just test-devtools           # quoin-devtools suite
just test-headless-examples  # todo-core + fetch-core + devtools-demo e2e
```

### Testing Your Own Code (quoin-testkit)
Test framework-agnostic hooks and stores headlessly — no window manager, no
browser, fully deterministic:

```rust
use quoin_core::prelude::*;
use quoin_testkit::{ImmediateExecutor, TestContext};

fn use_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
    cx.create_signal(0u32)
}

let cx = TestContext::with_executor(ImmediateExecutor);
let count = use_counter(&cx);
count.set(41);
count.update(|v| *v += 1);
assert_eq!(count.get(), 42);
assert_eq!(cx.updates().count(), 2);  // exact re-render churn
```

Pick `ImmediateExecutor` for deterministic state machines,
`ThreadedExecutor` for real concurrency and abort semantics, or
`TokioExecutor` (feature `tokio`) inside `#[tokio::test]`.

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

### Debugging the Dev Loop (quoin-devtools)
Wrap **any** adapter context and see what your app is doing — signal
creations, mutations, update requests, task spawns/completions/aborts.
See it live in a real window: the `counter-xilem` demo renders the trace
counts next to the counter and dumps the session when the window closes.

```rust,ignore
use quoin_devtools::Instrumented;
let ctx = Instrumented::wrap(raw_ctx, "app");   // drop-in wrapper
```

```bash
QUOIN_TRACE=1 ./myapp          # enables recording from the environment
```

Then inspect (from a debug hotkey, a drop-down panel, or on exit):
```rust,ignore
quoin_devtools::global_log().dump();                 // stderr table
quoin_devtools::global_log().hot_signals();          // ranked mutation counts
quoin_devtools::global_log().events();               // raw snapshot
```

The zero-cost gate means shipping this wiring is safe: with tracing disabled
each call is one relaxed atomic load and nothing is allocated. The optional
`tracing` feature forwards events into the `tracing` ecosystem.

### Community & contribution files
`.github/` ships the usual contribution plumbing: issue templates
(`bug_report.yml`, `feature_request.yml`), a pull-request template,
`dependabot.yml` (workspace + standalone example manifests + Actions),
`CODEOWNERS`, and `FUNDING.yml`. The changelog is generated from Conventional
Commits by [git-cliff](https://git-cliff.org) (`just changelog`).

### CI / CD & Releases
CI runs on GitHub Actions with the latest action versions (`actions/checkout@v7`, `Swatinem/rust-cache@v2`, `taiki-e/install-action@v2`):

| Workflow | Trigger | What it does |
| --- | --- | --- |
| `ci.yml` | push / PR to `main` | Format check, clippy (`-D warnings`), core + testkit + devtools tests, adapter conformance tests, trybuild macro UI tests (Leptos, Dioxus, GPUI), headless example e2e, full example build |
| `release.yml` | tag `v*.*.*` | Version-tag sanity check, full test suite, `git-cliff` changelog generation, GitHub Release with generated notes |

### Lint policy: pedantic, zero suppressions
The workspace defines its lint policy **once** in the root `Cargo.toml`
(`[workspace.lints]`, inherited by every crate via `[lints] workspace = true`):

- `clippy::pedantic` + `clippy::all` + `missing_docs` — all warnings, and CI
  runs with `-D warnings` so they are hard errors.
- `unsafe_code = "forbid"` everywhere.
- `clippy::allow_attributes = "deny"` — `#[allow(...)]` attributes are banned
  outright. Issues are fixed at the source (or gated with `#[cfg]`), never
  suppressed.

### Benchmarks
Micro-benchmarks run with a plain `std::time::Instant` harness (no external
benchmark framework), so they work everywhere:

```bash
just bench-core                                        # signals, derived, memo, cancellation, pool
just bench-devtools                                    # event log recording + aggregate views
```

Numbers are machine-relative regression baselines — compare across commits on
the same machine, not across hardware. Reference numbers and methodology live
in [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md).

#### Changelog (`git-cliff`)
`CHANGELOG.md` is generated with [git-cliff](https://git-cliff.org) from [Conventional Commits](https://www.conventionalcommits.org/).
The config (`cliff.toml`) is deliberately emoji-free; commit prefixes map to
plain-text groups:
- `feat:` → Added - New features and capabilities
- `fix:` → Fixed - Bug fixes and corrections
- `perf:` → Performance - Optimizations and improvements
- `refactor:` → Changed - Refactoring
- `doc:` → Documentation - Documentation changes
- `test:` → Tests - Test additions and changes
- `chore:` → Cleanup - Code maintenance and housekeeping
- `ci:` / `build:` / `style:` / `revert:` → their own named groups
- breaking changes (`!:`) are always preserved

Regenerate the changelog locally with:
```bash
just changelog        # rewrite CHANGELOG.md from the commit history
just release-notes    # print the notes for the latest version only
```
To cut a release, bump the crate versions, tag `vX.Y.Z`, and push the tag — the `release.yml` workflow verifies tag/version consistency, runs the full suite, and publishes the release with git-cliff notes.

## Contributing
We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Setting up the development environment
- Adding a new framework adapter
- Extending the macro syntax
- Running and writing tests

Before submitting a pull request, ensure:
- All tests pass: `just test`
- Code is formatted: `just fmt`
- No clippy warnings: `just check`
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `doc:`, …) — they drive the git-cliff changelog.

## License
Quoin is licensed under the terms of the MIT License.  
See [LICENSE.md](LICENSE.md) for details.
