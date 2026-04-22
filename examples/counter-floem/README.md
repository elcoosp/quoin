# counter-floem

Minimal counter app using Floem 0.2 and `quoin`.

## Run

 ```bash
just run-floem
# or
cargo run --manifest-path examples/counter-floem/Cargo.toml
 ```

## How it works

1. `FloemContext::new()` creates the reactive context.
2. `label(move || ...)` and `button(...).action(...)` use Floem's reactive
   closures which automatically track `Signal::get()` calls.
3. Signals are backed by `floem_reactive::RwSignal` wrapped in
   `SendWrapper` for thread safety.
