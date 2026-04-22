# counter-xilem

Minimal counter app using Xilem 0.4 and `quoin`.

## Run

 ```bash
just run-xilem
# or
cargo run --manifest-path examples/counter-xilem/Cargo.toml
 ```

## How it works

1. A `tokio::runtime::Runtime` is created and wrapped in `Arc`.
2. `XilemContext::new(runtime)` holds the runtime and an optional
   update notifier.
3. Signals use `Arc<RwLock<T>>` — mutations call `request_update()`
   which triggers a full Xilem rebuild.
4. `set_update_notifier` can be used to request incremental updates
   (currently just logs).

> **Note:** Xilem uses an immediate-mode rebuild model. Every signal
> mutation schedules a full view rebuild via the `Xilem` state callback.
