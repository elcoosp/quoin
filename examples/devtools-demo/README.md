# devtools-demo

Headless demo of **`quoin-devtools`**: runs a simulated UI session against an
`Instrumented` context and prints the recorded event log.

## Run it

```bash
just trace-demo
# equivalent to:
QUOIN_TRACE=1 cargo run -p devtools-demo
```

Sample output:

```text
simulated session: 3 items
== quoin-devtools (15 events retained) ==
  cart/created      3
  cart/mutated      9
  cart/update      1     <- the batched price update collapsed 3 writes into 1
  cart/spawn        1
  cart/done         1
hot signals (mutations by payload type):
  Vec<String>     4
  Vec<u32>        3
```

## Why this matters

`Instrumented` wraps the `ReactiveContext` **trait**, so the identical
instrumentation works with GPUI, Leptos, Dioxus, Xilem, Floem, and the
testkit. Point it at your real adapter context during development, run with
`QUOIN_TRACE=1`, and dump/inspect the log to find churning signals, redundant
re-render requests, and abandoned tasks.
