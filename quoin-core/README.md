# quoin-core

[![Docs.rs](https://docs.rs/quoin-core/badge.svg)](https://docs.rs/quoin-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](#license)

**The framework-agnostic reactive core of the quoin ecosystem.**

`quoin-core` defines the traits every UI framework adapter implements, plus a
growing set of adapter-agnostic building blocks layered on top via blanket
impls — so one set of types works identically on GPUI, Leptos, Dioxus, Xilem,
and Floem.

## ✨ What is inside

| Area | API | Notes |
|------|-----|-------|
| Core traits | `ReactiveContext`, `Signal<T>`, `Executor`, `JoinHandle<T>` | The contract every adapter implements. |
| Extensions | `SignalExt`, `ExecutorExt`, `ReactiveContextExt` | Blanket impls: `set_eq`, `replace`/`take`, `map`, `spawn_with_token`, `create_resource`, `batch`, `update_global`, `use_global_or_init`. |
| Derived signals | `derived()`, `Derived`, `memo()`, `Memo`, `zip()`, `Zipped` | Computed views recomputed on read; `memo` caches until the source changes. |
| Effects | `create_effect()`, `watch()`, `EffectHandle`, `EffectScope`, `TrackedSignal`, `DepHandle` | Push-based reactive effects with automatic dependency tracking — complete the reactive triangle portably (see below). |
| Erased signals | `AnySignal<T>` | ADR-002's opt-in dynamic-dispatch wrapper for storing heterogeneous signal handles. |
| Resources | `Resource`, `ResourceState` | `Idle → Loading → Ready(T) / Failed(E)` with race-free reloads and cooperative cancellation. |
| Batching | `batch` module, `BatchGuard` | Suppress redundant re-render requests inside a scope. |
| Local tasks | `LocalExecutor`, `LocalJoinHandle`, `LocalTask`, `spawn_local_task`, `run_local_tasks` | `!Send` futures: platform-driven on WASM, deterministic thread-local queue on native. |
| Worker pool | `pool::spawn_pooled`, `pool::block_on` | Shared growable thread pool backing the native adapters. |
| Cancellation | `CancellationToken`, `Cancelled`, `CancelledFuture`, `CancelGuard`, `RunUntilCancelled` | `child_token()`, `guard()`, `into_cancelled()`, `run_until_cancelled()`. |
| Test helpers | `test_support` module | Minimal `StaticContext`/`StaticSignal`/`StaticExecutor` for doctests; use `quoin-testkit` for real test suites. |

`#![forbid(unsafe_code)]` — the entire crate is safe Rust.

## 🔁 Reactive effects (`create_effect` / `watch`)

The derived layer is *pull-based* (recompute on read). Effects are the push
side — "when any signal I read changes, run this" — implemented once in the
core so they work headlessly and on every tracked-signal adapter:

```rust
use quoin_core::prelude::*;

let query = quoin_core::test_support::static_signal("rust".to_string());
let runs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

let (query2, runs2) = (query.clone(), runs.clone());
let handle = create_effect(move || {
    let q = query2.get();                 // read = dependency
    runs2.fetch_add(q.len(), std::sync::atomic::Ordering::Relaxed);
});
// Initial run happened; `query.set(...)` re-runs it. Batched writes
// coalesce into one re-run; `handle.dispose()` stops the effect;
// `EffectScope` disposes groups of effects on drop (component lifetime).
```

Semantics: dynamic dependency sets rebuilt per run, batch-aware flush, FIFO
propagation, deferred self-writes (no recursion), detached-by-default
handles owned by the runtime until disposed. Reads outside effect runs pay
one thread-local flag check — see `docs/BENCHMARKS.md`.

## 📦 Usage

Application code usually depends on the `quoin` facade instead; `quoin-core`
is for library authors writing framework-agnostic hooks and stores:

```toml
[dependencies]
quoin-core = "0.1"
```

```rust
use quoin_core::prelude::*;

fn use_counter<C: ReactiveContext>(cx: &C) -> impl Signal<String> {
    cx.create_signal(0u32).map(|n| format!("count = {n}"))
}
```

## 🧩 Core Abstractions

| Trait | Purpose |
|-------|---------|
| **`ReactiveContext`** | Creates signals and provides the async executor. |
| **`Signal<T>`** | Readable and writable reactive value (`get`/`with`/`set`/`update`). |
| **`Executor`** | Spawns `Send` futures on the framework's native runtime. |
| **`JoinHandle<T>`** | Abort / detach / drop-cancels (structured concurrency). |
| **`LocalExecutor`** | Spawns `!Send` futures (WASM main thread / native queue). |
| **`CancellationToken`** | Cooperative cancellation with child tokens and RAII guards. |

## 🧪 Testing

This crate is exercised by:

- 45+ unit tests and 18 doctests (`cargo test -p quoin-core`),
- the shared conformance suite in `quoin-conformance` (30+ checks per adapter),
- the e2e examples `examples/todo-core` and `examples/fetch-core`,
- micro-benchmarks: `cargo run --release -p quoin-core --example bench`.

## 🤝 Contributing

Contributions are welcome! See the repository-level
CONTRIBUTING.md and the conformance suite when adding
API.

## 📄 License

Licensed under the MIT license.
