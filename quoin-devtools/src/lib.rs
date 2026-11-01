//! # quoin-devtools — see what your reactive app is doing
//!
//! A framework-agnostic instrumentation layer for quoin. Wrap **any**
//! adapter's `ReactiveContext` (GPUI, Leptos, Dioxus, Xilem, Floem, or the
//! `quoin-testkit` context) and every signal creation, read-triggering
//! mutation, task spawn, and UI update request is recorded into an
//! inspectable [`EventLog`].
//!
//! ```text
//! use quoin_devtools::Instrumented;
//!
//! let raw_ctx = /* any adapter context */;
//! let ctx = Instrumented::wrap(raw_ctx, "app");
//! // use `ctx` everywhere you used `raw_ctx`
//! ```
//!
//! ## Reading the log
//!
//! ```text
//! quoin_devtools::global_log().dump();              // stderr table
//! for event in quoin_devtools::global_log().events() { … }
//! ```
//!
//! ## Zero-cost when disabled
//!
//! Unless [`enabled()`] is turned on (default: the `QUOIN_TRACE=1`/`QUOIN_TRACE=on`
//! environment variable, or an explicit [`enable()`] call), the `Instrumented`
//! wrapper short-circuits every recording call behind one relaxed atomic
//! load, and no events are allocated.
//!
//! ## Layout
//!
//! - [`event`] — [`Event`], [`EventKind`], and the ring-buffer [`EventLog`]
//!   with its aggregate views.
//! - [`record`] — the global log, the `QUOIN_TRACE` bootstrap, and the
//!   recording helpers shared by every wrapper.
//! - [`instrumented`] — the `Instrumented` context, `TracedSignal`, and
//!   `TracedExecutor`.

pub mod event;
pub mod instrumented;
pub mod record;

pub use event::{Event, EventKind, EventLog};
pub use instrumented::{Instrumented, TracedExecutor, TracedSignal};
pub use record::{
    disable, enable, enabled, global_log, init_from_env, parse_trace_flag, record, snapshot,
    thread_label,
};

/// How many events the default log keeps before overwriting the oldest.
pub const DEFAULT_LOG_CAPACITY: usize = 4096;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexports_are_reachable() {
        // Compile-time smoke test for the public re-export surface.
        assert_eq!(EventKind::UpdateRequested.label(), "update");
        assert_eq!(DEFAULT_LOG_CAPACITY, 4096);
        let log = EventLog::with_capacity(4);
        assert!(log.is_empty());
        assert_eq!(global_log().capacity(), DEFAULT_LOG_CAPACITY);
    }
}
