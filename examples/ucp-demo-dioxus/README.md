# ucp-demo-dioxus

UCP demo app running the shared `DemoApp` component on Dioxus 0.7 (desktop).

## Run

 ```bash
just run-ucp-dioxus
# or
cd examples/ucp-demo-dioxus && cargo run
 ```

## Styling

Tailwind CSS is loaded via CDN in a custom head injected through
 `Config::with_custom_head`. This provides the utility classes used in
 `quoin_render!` without a build-time Tailwind step.
