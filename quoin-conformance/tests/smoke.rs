//! Compile-time smoke test: the core traits keep their documented bounds.
use quoin_core::prelude::*;

const fn assert_traits<T: ReactiveContext + Clone + Send + Sync>() {}
const fn assert_executor<E: Executor + Clone + Send + Sync>() {}
const fn assert_signal<S: Signal<u32> + Clone + Send + Sync>() {}

#[test]
fn trait_bounds_compile() {
    // This test simply compiles; it verifies the trait bounds are consistent.
    // Referencing the functions keeps them from being flagged as unused.
    assert_traits::<quoin_testkit::TestContext>();
    let _ = assert_executor::<quoin_testkit::ThreadedExecutor>;
    let _ = assert_signal::<quoin_testkit::TestSignal<u32>>;
}
