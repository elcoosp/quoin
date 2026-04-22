# counter-gpui

Minimal counter app using GPUI (Zed's UI framework) and `quoin`.

## Run

 ```bash
just run-gpui
# or
cargo run --manifest-path examples/counter-gpui/Cargo.toml
 ```

## How it works

1. `GpuiContext` is created from the GPUI `Context` via `cx.into()`.
2. `set_view_update_notifier` wires signal mutations to `cx.notify()`.
3. `use_counter` (from `counter-lib`) returns a `GpuiSignal<u32>` that
   automatically re-renders the view on every `update()`.
4. The `run_app!` macro generates the `main()` boilerplate.
