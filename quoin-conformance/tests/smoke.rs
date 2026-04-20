#![allow(dead_code)]
use quoin::{Executor, ReactiveContext, Signal};
fn assert_traits<T: ReactiveContext + Clone + Send + Sync>() {}
fn assert_executor<E: Executor + Clone + Send + Sync>() {}
fn assert_signal<S: Signal<u32> + Clone + Send + Sync>() {}

#[test]
fn trait_bounds_compile() {
    // This test simply compiles; it verifies the trait bounds are consistent.
}
