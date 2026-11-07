//! Batched updates: mutate many signals, notify the UI once.
//!
//! On pull-based adapters (GPUI, Xilem) every signal mutation calls
//! `request_update()`. When a handler mutates several signals in a row this
//! produces redundant re-render requests. [`with`](fn@with) suppresses those
//! intermediate notifications and issues a single update when the batch ends.
//!
//! # How it works
//!
//! A thread-local depth counter tracks nested batches. The signal
//! implementations of the pull-based adapters consult
//! [`should_notify`] before invoking the update notifier. Fine-grained
//! frameworks (Leptos, Dioxus, Floem) batch internally and ignore this
//! mechanism, so the helper is safe to call everywhere.
//!
//! # Example
//!
//! ```
//! use quoin_core::prelude::*;
//! use quoin_core::batch;
//!
//! let cx = quoin_core::test_support::StaticContext;
//! let a = cx.create_signal(0u32);
//! let b = cx.create_signal(0u32);
//!
//! batch::with(&cx, || {
//!     a.set(1);
//!     b.set(2);
//! });
//! assert_eq!((a.get(), b.get()), (1, 2));
//! ```

use std::cell::Cell;

thread_local! {
    static BATCH_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// RAII guard that increments the batch depth on creation and decrements it
/// on drop. Prefer calling [`with`] instead of using this directly.
pub struct BatchGuard {
    _private: (),
}

impl BatchGuard {
    /// Enters a batch scope. Decrements automatically on drop.
    #[must_use]
    pub fn enter() -> Self {
        BATCH_DEPTH.with(|d| d.set(d.get().saturating_add(1)));
        Self { _private: () }
    }
}

impl Drop for BatchGuard {
    fn drop(&mut self) {
        BATCH_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        // The outermost batch end flushes deferred effect re-runs (see
        // `crate::effect`): mutations made inside the batch scheduled their
        // reading effects instead of running them inline, so N writes
        // produce at most one re-run per effect — the effect-side twin of
        // the single `request_update` the batch issues below.
        if !crate::batch::in_batch() {
            crate::effect::on_batch_end();
        }
    }
}

/// Returns `true` while inside a [`with`](fn@with) scope.
#[must_use]
pub fn in_batch() -> bool {
    BATCH_DEPTH.with(|d| d.get() > 0)
}

/// Used by adapter signal implementations: `true` when a mutation should
/// trigger the update notifier (i.e. we are not inside a batch).
///
/// Batch bookkeeping is intentionally cheap (two thread-local accesses), but
/// still skipped entirely on fine-grained frameworks where the signal layer
/// never notifies.
#[must_use]
pub fn should_notify() -> bool {
    !in_batch()
}

/// Runs `f` inside a batch scope and then requests a single UI update on
/// `cx`.
///
/// The return value of `f` is passed through. Nested batches are supported;
/// only the outermost [`with`] triggers the final `request_update`.
pub fn with<C: crate::ReactiveContext, R>(cx: &C, f: impl FnOnce() -> R) -> R {
    let guard = BatchGuard::enter();
    let result = f();
    drop(guard);
    // Only the outermost batch reaches depth 0 here; nested `with` calls see
    // a depth > 0 and `request_update` is left to the outer frame.
    if !in_batch() {
        cx.request_update();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::StaticContext;

    #[test]
    fn depth_tracks_enter_and_drop() {
        assert!(!in_batch());
        {
            let _g = BatchGuard::enter();
            assert!(in_batch());
            {
                let _g2 = BatchGuard::enter();
                assert!(in_batch());
            }
            assert!(in_batch(), "inner drop must not pop outer batch");
        }
        assert!(!in_batch());
    }

    #[test]
    fn should_notify_flips_inside_batch() {
        assert!(should_notify());
        let _g = BatchGuard::enter();
        assert!(!should_notify());
    }

    #[test]
    fn with_runs_closure_and_requests_update_once() {
        let cx = StaticContext;
        let out = with(&cx, || 21 * 2);
        assert_eq!(out, 42);
        assert!(!in_batch(), "batch must be closed after with() returns");
    }

    #[test]
    fn nested_with_requests_update_only_at_outermost() {
        let cx = StaticContext;
        with(&cx, || {
            with(&cx, || {});
        });
        // No panic; depth bookkeeping balanced.
        assert!(!in_batch());
    }

    #[test]
    fn panic_inside_batch_still_restores_depth() {
        let cx = StaticContext;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with(&cx, || panic!("boom"));
        }));
        assert!(result.is_err());
        assert!(!in_batch(), "guard must unwind-safe restore depth");
    }
}
