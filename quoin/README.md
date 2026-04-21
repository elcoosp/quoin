# quoin

[![Crates.io](https://img.shields.io/crates/v/quoin.svg)](https://crates.io/crates/quoin)
[![Docs.rs](https://docs.rs/quoin/badge.svg)](https://docs.rs/quoin)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**One reactive core, many frameworks.**

`quoin` is the facade crate for the quoin ecosystem. It re-exports the core reactive
traits from `quoin-core` and, when a framework feature is enabled, also re-exports
the corresponding adapter types and macros — all from a single dependency.

## 📦 Usage

Add `quoin` to your `Cargo.toml` and enable **exactly one** adapter feature:

```toml
[dependencies]
quoin = { version = "0.1", features = ["gpui"] }   # or "leptos", "dioxus", "floem", "xilem"
