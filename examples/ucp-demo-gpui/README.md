# ucp-demo-gpui

UCP demo app running the shared `DemoApp` component on GPUI.

## Run

 ```bash
just run-ucp-gpui
# or
cargo run --manifest-path examples/ucp-demo-gpui/Cargo.toml
 ```

## What you'll see

- Dark-themed layout with Tailwind classes transpiled to GPUI style chains
- Increment button with reactive count display
- Option selector (A / B) buttons
- People list rendered from a `Vec<Person>` signal

The entire UI is defined in `ucp-lib` using `component!` and `quoin_render!`.
This crate only contains the `run_app!` bootstrap.
