
# quoin-dioxus

[![Crates.io](https://img.shields.io/crates/v/quoin-dioxus.svg)](https://crates.io/crates/quoin-dioxus)
[![Docs.rs](https://docs.rs/quoin-dioxus/badge.svg)](https://docs.rs/quoin-dioxus)

Dioxus adapter for [quoin](https://crates.io/crates/quoin).

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
quoin = "0.1"
quoin-dioxus = "0.1"
```

Use `DioxusContext` in your components:

```rust
use dioxus::prelude::*;
use quoin::ReactiveContext;
use quoin_dioxus::DioxusContext;

fn App() -> Element {
    let ctx = DioxusContext::new();
    let signal = ctx.create_signal(0u32);

    rsx! {
        div { "Count: {signal.get()}" }
    }
}
```

## Conformance

This adapter passes the `quoin-conformance` test suite.

## License

MIT OR Apache-2.0
