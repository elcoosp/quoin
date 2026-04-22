# counter-leptos

Minimal counter app using Leptos 0.8 with SSR support and `quoin`.

## Run

### Client-side only (WASM)

 ```bash
cd examples/counter-leptos && trunk serve
 ```

### Full SSR

 ```bash
just run-leptos
# or
cargo leptos serve -p counter-leptos
 ```

The server binds to http://127.0.0.1:3000 by default.

## Build artifacts

| Mode | Entry | Description |
|------|-------|-------------|
| WASM | `src/lib.rs` | wasm_bindgen start mounts the app |
| SSR  | `src/main.rs` | Axum server with leptos_axum integration |

## How it works

- `LeptosContext::new()` creates the reactive context.
- `Signal::get()` is brought into scope via `use quoin::Signal` to
  disambiguate from Leptos' own signal methods.
- The `index.html` uses `data-trunk` attributes for Trunk bundling.
