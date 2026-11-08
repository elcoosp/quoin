//! Effect conformance suite executed against the testkit `TestContext`.
//!
//! The testkit's `TestSignal` is the reference implementation of a tracked
//! pull-based signal (the same shape as the GPUI/Xilem backing cells), so
//! this suite pins the core effect runtime contract end-to-end through a
//! real [`ReactiveContext`]. Adapters whose signals implement
//! [`TrackedSignal`](quoin_core::TrackedSignal) wire the identical checks
//! with [`define_effect_tests!`](quoin_conformance::define_effect_tests).

use quoin_conformance::{TestContextProvider, define_effect_tests};
use quoin_core::ReactiveContext;
use quoin_testkit::{ImmediateExecutor, TestContext};

/// Harness over the testkit context: tracked signals, immediate executor.
#[derive(Clone)]
struct EffectHarness(TestContext<ImmediateExecutor>);

impl TestContextProvider for EffectHarness {
    fn setup_context() -> Self {
        Self(TestContext::with_executor(ImmediateExecutor))
    }
}

impl ReactiveContext for EffectHarness {
    type Signal<T: Clone + Send + 'static> =
        <TestContext<ImmediateExecutor> as ReactiveContext>::Signal<T>;
    type Executor = <TestContext<ImmediateExecutor> as ReactiveContext>::Executor;

    fn create_signal<T: Clone + Send + 'static>(&self, initial: T) -> Self::Signal<T> {
        self.0.create_signal(initial)
    }

    fn executor(&self) -> Self::Executor {
        self.0.executor()
    }

    fn request_update(&self) {
        self.0.request_update();
    }

    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T) {
        self.0.provide_global(value);
    }

    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        self.0.use_global::<T>()
    }
}

define_effect_tests!(EffectHarness);
