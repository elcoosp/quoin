# ucp-lib

Shared Universal Component Protocol (UCP) component library.

Provides `component!`-based components that compile to GPUI, Leptos, or
Dioxus depending on the enabled Cargo feature.

## Features

| Feature | Enables |
|---------|---------|
| gpui   | GPUI adapter + quoin-macros/gpui |
| leptos | Leptos adapter + quoin-macros/leptos |
| dioxus | Dioxus adapter + quoin-macros/dioxus |

## Components

### DemoApp

A showcase component with:

- Signal state (`count`, `selected`)
- Vector state (`rows: Vec<Person>`) with iteration via `for[...]`
- Inline event handlers (`on_click: |_| ...`)
- **`quoin_render!`** blocks with Tailwind-like class names

### MiniDevtools

A Chrome-DevTools-style panel with:

- Tabbed interface (`tabs` / `tab` elements)
- Data table (`data_table` / `column`) with striped rows
- Conditional rendering (`if[condition] { ... } else if { ... }`)
- Text filtering with reactive `input` binding
- Signal inspection tab listing active state

## Usage

 ```toml
[dependencies]
ucp-lib = { path = "../ucp-lib", features = ["gpui"] }
quoin   = { path = "../../quoin", features = ["gpui"] }
 ```

 ```rust
use ucp_lib::DemoApp;
run_app!(DemoApp);
 ```

## Consumers

| Crate              | Framework |
|--------------------|-----------|
| ucp-demo-gpui      | GPUI      |
| ucp-demo-leptos    | Leptos    |
| ucp-demo-dioxus    | Dioxus    |
| mini-devtools-gpui | GPUI + gpui-component |
