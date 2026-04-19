
# quoin-gpui

[![Crates.io](https://img.shields.io/crates/v/quoin-gpui.svg)](https://crates.io/crates/quoin-gpui)
[![Docs.rs](https://docs.rs/quoin-gpui/badge.svg)](https://docs.rs/quoin-gpui)

GPUI adapter for [quoin](https://crates.io/crates/quoin).

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
quoin = "0.1"
quoin-gpui = "0.1"
```


Obtain a `GpuiContext` from your component:

```rust
use gpui::*;
use quoin::ReactiveContext;
use quoin_gpui::GpuiContext;

struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ctx = GpuiContext::new(cx);
        let signal = ctx.create_signal(self.count);

        div()
            .child(format!("Count: {}", signal.get()))
    }
}
```

## Conformance

This adapter passes the `quoin-conformance` test suite.

## License

MIT OR Apache-2.0
