# mini-devtools-gpui

A mini devtools panel built with GPUI, `gpui-component`, and `quoin`.

Demonstrates integrating quoin's `component!` / `quoin_render!` macros
with the `gpui-component` widget library (buttons, inputs, tabs, etc.).

## Run

 ```bash
just run-mini-devtools
# or
cargo run --manifest-path examples/mini-devtools-gpui/Cargo.toml

# Watch mode
just watch-mini-devtools
 ```

## Features

- Tabbed interface: Timeline, Cache, Signals tabs
- Event filtering: reactive text input filters timeline events
- Data table: cache entries displayed in a striped table
- Signal inspection: shows active signal names and values
- Add Event button: increments an event counter

## Architecture

 ```text
mini-devtools-gpui (main.rs)
  -> ucp-lib::MiniDevtools  (component! macro)
       -> quoin_render!      (declarative UI)
            -> quoin-ui-gpui (QuoinInputManager, theme, etc.)
 ```

The `MiniDevtools` component is defined in `ucp-lib` so it can be
shared across frameworks in the future.
