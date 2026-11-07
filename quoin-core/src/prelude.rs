//! Common imports for trait-only consumers of `quoin-core`.
//!
//! Covers the core traits **plus** the blanket extension traits, so one
//! import line unlocks the full framework-agnostic API:
//!
//! ```text
//! use quoin_core::prelude::*;
//!
//! let cx = /* any adapter context */;
//! let count = cx.create_signal(0u32);
//! let label = count.map(|n| format!("count = {n}"));   // SignalExt
//! let res = cx.create_resource(fetch);                  // ReactiveContextExt
//! let (handle, token) = cx.executor().spawn_with_token(|t| async move { … });
//! ```

pub use crate::batch;
pub use crate::{
    AnySignal, CancelGuard, CancellationToken, Cancelled, CancelledFuture, Derived, Executor,
    ExecutorExt, JoinHandle, LocalExecutor, LocalJoinHandle, LocalTask, Memo, ReactiveContext,
    ReactiveContextExt, Resource, ResourceState, RunUntilCancelled, Signal, SignalExt, SleepFuture,
    SyncSleepFuture, create_effect, derived, memo, sleep, watch,
};
