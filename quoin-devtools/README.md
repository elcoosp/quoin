# quoin-devtools

Framework-agnostic devtools for [quoin](https://github.com/elcoosp/quoin): see what your
reactive app is doing while you develop.

## What you get

- **`Instrumented<C>`** — wraps *any* adapter's `ReactiveContext` (GPUI, Leptos, Dioxus,
  Xilem, Floem, or `quoin-testkit`) and records signal creations, mutations, `request_update`
  calls, and task spawn/complete/abort events.
- **`EventLog`** — a bounded ring buffer of events with aggregate views:
  `summary()` (per-scope counts), `hot_signals()` (ranked mutation counts per payload type),
  `events()` (raw snapshot), `dump()` (stderr table).
- **`snapshot(signal, scope)`** — record a mutation with the actual `Debug` value attached.
- **Zero-cost when off** — recording is gated behind one relaxed atomic load; nothing is
  allocated while disabled.

## Quick start

```toml
[dependencies]
quoin-devtools = "0.1"
```

```rust,ignore
use quoin_devtools::Instrumented;

fn main() {
    quoin_devtools::init_from_env(); // QUOIN_TRACE=1 ./myapp

    let raw_ctx = /* any adapter context */;
    let ctx = Instrumented::wrap(raw_ctx, "app");
    // ...use `ctx` exactly like `raw_ctx`...
}
```

Run with tracing enabled and dump at exit:

```bash
QUOIN_TRACE=1 ./myapp; quoin_devtools::global_log().dump();
```

Or inspect programmatically:

```rust,ignore
for (scope, count) in quoin_devtools::global_log().summary() {
    println!("{scope}: {count}");
}
println!("{:?}", quoin_devtools::global_log().hot_signals());
```

## Dev loop integration

- `just trace-<example>` — run an example with `QUOIN_TRACE=1`.
- Pair with `quoin-testkit` to assert on update churn in tests (see the testkit README).
- The optional `tracing` feature forwards events to the `tracing` ecosystem
  (`tracing::trace!` on target `quoin_devtools`).

