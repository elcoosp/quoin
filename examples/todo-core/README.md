# todo-core

A complete, framework-agnostic **todo application store** — the "app brain" of a
todo app, written once against quoin's traits and e2e tested headlessly.

The identical store can be wired into GPUI, Leptos, Dioxus, Xilem, or Floem views
(see `examples/counter-*` for view wiring patterns).

## What it demonstrates

| Feature | Where |
| --- | --- |
| Owned reactive state | `todos`, `filter`, `draft` signals |
| Multi-signal transactions | `commit_draft` runs inside one `quoin_core::batch` (1 re-render, not 4) |
| No-op write suppression | `set_draft` / `set_filter` / `set_all_completed` use `SignalExt::set_eq` |
| Computed views | `footer_text()` is a `derived` signal |
| Scoped async work | `SaveIndicator` uses `CancellationToken::child_token` |

## Run the e2e suite

```bash
cargo test -p todo-core
```

`tests/e2e.rs` drives full user sessions through `quoin-testkit`:
type-commit-toggle-filter-clear flows, render-churn assertions (no-op
interactions produce **zero** re-render requests), batch-transaction
assertions (one commit == exactly one update request), and a 100-item stress
pass on the threaded executor.
