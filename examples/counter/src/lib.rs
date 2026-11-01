//! Framework-agnostic counter hook shared by every visual counter demo.
//!
//! The same `use_counter` code runs unchanged on GPUI, Floem, Xilem, and the
//! headless testkit — that is the whole quoin pitch in one file.

use quoin_core::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

/// Counter state plus its increment action.
///
/// The closure pointer type is generic (`Rc` for single-threaded adapters,
/// `Arc` when the app state must be `Send`), so every framework can reuse
/// this struct instead of hand-rolling its own counter.
#[derive(Clone)]
pub struct Counter<S: Signal<u32>, F = Rc<dyn Fn()>> {
    /// The reactive count value.
    pub count: S,
    /// Action that increments the count.
    pub increment: F,
}

/// Creates a counter hook on any reactive context (single-threaded flavor).
///
/// Returns a `Counter` whose action is an `Rc` closure — the natural choice
/// for adapters whose app state is not required to be `Send` (GPUI, Floem).
pub fn use_counter<C: ReactiveContext>(cx: &C) -> Counter<C::Signal<u32>> {
    let count = cx.create_signal(0u32);
    let increment: Rc<dyn Fn()> = Rc::new(increment_action(&count));
    Counter { count, increment }
}

/// Creates a counter hook whose action is an `Arc` closure.
///
/// Use this when the surrounding app state must be `Send + Sync` (Xilem,
/// server-side Leptos). Same logic, thread-safe action pointer.
pub fn use_counter_send<C: ReactiveContext>(
    cx: &C,
) -> Counter<C::Signal<u32>, Arc<dyn Fn() + Send + Sync>>
where
    C::Signal<u32>: Send + Sync + 'static,
{
    let count = cx.create_signal(0u32);
    let increment: Arc<dyn Fn() + Send + Sync> = Arc::new(increment_action_send(&count));
    Counter { count, increment }
}

/// Builds the increment action for a signal (single-threaded wrapper target).
fn increment_action<S>(count: &S) -> Box<dyn Fn()>
where
    S: Signal<u32> + 'static,
{
    let count = count.clone();
    Box::new(move || count.update(|c| *c += 1))
}

/// Builds the increment action for a `Send + Sync` signal.
fn increment_action_send<S>(count: &S) -> Box<dyn Fn() + Send + Sync>
where
    S: Signal<u32> + Send + Sync + 'static,
{
    let count = count.clone();
    Box::new(move || count.update(|c| *c += 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_testkit::{ImmediateExecutor, TestContext};

    #[test]
    fn counter_increments_via_action() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let counter = use_counter(&cx);
        (counter.increment)();
        (counter.increment)();
        assert_eq!(counter.count.get(), 2);
        cx.updates().assert_count(2);
    }

    #[test]
    fn send_counter_increments_via_action() {
        fn assert_send_sync<T: Send + Sync>() {}

        let cx = TestContext::with_executor(ImmediateExecutor);
        let counter = use_counter_send(&cx);
        (counter.increment)();
        assert_eq!(counter.count.get(), 1);
        // The action is Send + Sync, so it can cross threads.
        assert_send_sync::<Arc<dyn Fn() + Send + Sync>>();
    }
}
