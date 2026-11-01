# counter-xilem

Visual counter app using Xilem 0.4 and `quoin` — **with the quoin devtools
wired in**. It is the reference demo for the "debug the whole dev loop"
story: the reactive context is wrapped in `quoin_devtools::Instrumented`, so
every signal creation, mutation, and update request is recorded while you
use the window.

## Run

 ```bash
cargo run --manifest-path examples/counter-xilem/Cargo.toml
 ```

While the window is open:

- **Count** — the shared counter hook (`counter-lib::use_counter_send`),
  running on an `Instrumented<XilemContext>` instead of a bare context.
- **Increment / Reset** — both buttons mutate a `TracedSignal`, so each
  write lands in the devtools event log.
- **devtools line** — a live label showing how many events were recorded
  for this window (`created` / `mutated` / `update requests`).

When the window closes, the recorded session is dumped to stderr:

```text
== quoin-devtools (9 events retained) ==
  counter-xilem  created   1
  counter-xilem  mutated   7
  ...
```

## How it works

1. A `tokio::runtime::Runtime` is created and wrapped in `Arc`.
2. `XilemContext::new(runtime)` holds the runtime and an optional update
   notifier.
3. `Instrumented::wrap(ctx, "counter-xilem")` wraps the context; the counter
   hook is created from the wrapper, so its signals are `TracedSignal`s that
   record every mutation.
4. Signals use `Arc<RwLock<T>>` — mutations call `request_update()`, which
   triggers a full Xilem rebuild and refreshes both labels.
5. After `run_in` returns (window closed), `global_log().dump()` prints the
   recorded session.

> **Note:** Xilem uses an immediate-mode rebuild model. Every signal
> mutation schedules a full view rebuild via the `Xilem` state callback.

## Headless tests

The demo logic is exercised without a window by `cargo test` in this
directory: `run_session` simulates button presses against an instrumented
context and asserts on the recorded trace shape.

```bash
cargo test --manifest-path examples/counter-xilem/Cargo.toml
```
