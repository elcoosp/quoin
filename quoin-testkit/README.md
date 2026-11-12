# quoin-testkit

Deterministic testing toolkit for [quoin](https://github.com/elcoosp/quoin) reactive code.

Test framework-agnostic hooks, stores, and view models — written once against the
`ReactiveContext` trait — **without spinning up any UI framework**.

```toml
[dev-dependencies]
quoin-testkit = "0.1"
```

## What is in the box

| Component | Purpose |
|-----------|---------|
| `TestContext` | Full `ReactiveContext` with update counting and a **shared** global store. |
| `ImmediateExecutor` | Runs futures inline to completion — fully deterministic (blocking). |
| `ThreadedExecutor` | Runs each future on a dedicated thread — real concurrency and abort semantics. |
| `TokioExecutor` *(feature `tokio`)* | Runs futures on a tokio runtime; `TokioExecutor::global()` uses the ambient `#[tokio::test]` runtime. |
| `UpdateRecorder` | Assert exactly how many re-render requests were issued. |

## Example

```rust
use quoin_testkit::{TestContext, ImmediateExecutor};
use quoin_core::prelude::*;

fn use_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
    cx.create_signal(0u32)
}

let cx = TestContext::with_executor(ImmediateExecutor);
let count = use_counter(&cx);
count.set(41);
count.update(|v| *v += 1);
assert_eq!(count.get(), 42);
assert_eq!(cx.updates().count(), 2);
```

## Update counting

`TestContext` mirrors the pull-based adapters (GPUI, Xilem): every signal
mutation and every `request_update()` bumps the counter, while
`quoin_core::batch` collapses a group of mutations into a single counted
update. This lets tests assert on the *exact* UI churn an interaction
produces.

## Executor selection

- `ImmediateExecutor` — pure state machines; synchronous, deterministic, cannot
  model mid-flight cancellation.
- `ThreadedExecutor` — awaits timers/channels; real abort/detach semantics.
- `TokioExecutor` (feature `tokio`) — integration with tokio-based tests.

