# counter-dioxus

Minimal counter app using Dioxus 0.7 (desktop) and `quoin`.

## Run

 ```bash
just run-dioxus
# or
cd examples/counter-dioxus && cargo run
 ```

## How it works

1. `DioxusContext::new()` creates the reactive context inside a component.
2. Both the context and counter are stored via `use_hook` to persist
   across re-renders.
3. `counter.count.get()` reads the signal; `(counter.increment)()` calls
   the `Rc<dyn Fn()>` closure.

> **Note:** The `#[component]` macro from Dioxus is disambiguated in the
> source to avoid collision with quoin's own `component!` proc-macro.
