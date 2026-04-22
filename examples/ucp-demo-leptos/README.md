# ucp-demo-leptos

UCP demo app running the shared `DemoApp` component on Leptos 0.8 (WASM).

## Run

 ```bash
cd examples/ucp-demo-leptos && trunk serve
 ```

Opens a browser at http://127.0.0.1:8080 (Trunk default).

## Styling

Tailwind CSS v4 is loaded from CDN in `index.html`. The `style/tailwind.css`
file imports the Tailwind layer for potential future customization.

## Dependencies

| Package | Purpose |
|---------|---------|
| tailwindcss (dev) | v4 CSS layer import |
| trunk | WASM bundler (install separately) |

> **Note:** This example is CSR-only (no SSR server binary). Install
> Trunk with `cargo install trunk --locked`.
