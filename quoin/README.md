
# quoin

[![Crates.io](https://img.shields.io/crates/v/quoin.svg)](https://crates.io/crates/quoin)
[![Docs.rs](https://docs.rs/quoin/badge.svg)](https://docs.rs/quoin)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**One reactive core, many frameworks.**

`quoin` provides a foundational abstraction layer for building framework-agnostic
reactive libraries in Rust. Write your reactive logic once, and support GPUI,
Dioxus, Leptos, Xilem, Floem, and more—with only a feature flag toggle.

## Usage

Add `quoin` to your `Cargo.toml`:

```toml
[dependencies]
quoin = "0.1"
```

Define your library using the core traits:

```rust
use quoin::{ReactiveContext, Signal};

pub fn use_counter<C: ReactiveContext>(cx: &C) -> impl Signal<u32> {
    cx.create_signal(0)
}
```

Downstream users select a framework adapter via feature flags:

```toml
[dependencies]
my-agnostic-lib = { version = "1.0", features = ["gpui"] }
```

## Features

- `gpui`: Enables the GPUI adapter (requires `quoin-gpui`).
- `dioxus`: Enables the Dioxus adapter (requires `quoin-dioxus`).
- `leptos`: Enables the Leptos adapter (requires `quoin-leptos`).

**Only one adapter feature may be enabled at a time.**

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
