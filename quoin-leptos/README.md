
# quoin-leptos

[![Crates.io](https://img.shields.io/crates/v/quoin-leptos.svg)](https://crates.io/crates/quoin-leptos)
[![Docs.rs](https://docs.rs/quoin-leptos/badge.svg)](https://docs.rs/quoin-leptos)

Leptos adapter for [quoin](https://crates.io/crates/quoin).

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
quoin = "0.1"
quoin-leptos = "0.1"
```

Use `LeptosContext` in your components:

```rust
use leptos::*;
use quoin::ReactiveContext;
use quoin_leptos::LeptosContext;

#[component]
fn App() -> impl IntoView {
    let ctx = LeptosContext::new();
    let signal = ctx.create_signal(0u32);

    view! {
        <div>{move || signal.get()}</div>
    }
}
```


## Conformance

This adapter passes the `quoin-conformance` test suite.

## License

MIT OR Apache-2.0
