# fetch-core

Async data-loading with **`quoin_core::Resource`** — the framework-agnostic
equivalent of Leptos' `Resource` / Dioxus' `use_resource`, e2e tested.

## What it demonstrates

| Feature | Where |
| --- | --- |
| Async lifecycle (`Idle → Loading → Ready/Failed`) | `SearchPage.results` |
| Race-free reloads | `submit()` — a newer generation always wins; stale requests can't publish |
| Cooperative cancellation | `with_latency` races the delay against `token.cancelled()` |
| Typed errors | `FetchError::{Network, InvalidQuery}` surfaced via `ResourceState::Failed` |
| No-op write suppression | `set_query` uses `SignalExt::set_eq` |

## Run the e2e suite

```bash
cargo test -p fetch-core
```

`tests/e2e.rs` covers the deterministic paths with the
`ImmediateExecutor` (validation, error mapping, generation suppression) and
real concurrency with the `ThreadedExecutor` + latency backend (loading →
ready transitions, reload cancelling an in-flight request so only the newest
result publishes).

## Wiring into a real framework

```rust,ignore
// Leptos view (excerpt) — the same SearchPage drives the UI:
let page = SearchPage::new(&cx, "rust", http_backend());
view! {
    <input prop:value=page.query.get() on:input=move |e| page.set_query(event_target_value(&e)) />
    <button on:click=move |_| page.submit(&cx)>"Search"</button>
    {#if page.is_loading() } <p>"Loading…"</p> }
}
```
