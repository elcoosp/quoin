# counter-lib

Framework-agnostic counter hook built on `quoin-core`.

Demonstrates how to write reactive business logic once and reuse it
across any UI framework that implements `ReactiveContext`.

## Usage

 ```rust
use quoin_core::prelude::*;
use counter_lib::use_counter;

fn my_component<C: ReactiveContext>(cx: &C) {
    let counter = use_counter(cx);
    counter.count.update(|c| *c += 1);
    println!("count = {}", counter.count.get());
}
 ```

## See also

| Adapter | Crate |
|---------|-------|
| GPUI    | counter-gpui |
| Dioxus  | counter-dioxus |
| Leptos  | counter-leptos |
| Floem   | counter-floem |
| Xilem   | counter-xilem |
