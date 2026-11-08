//! Common imports for consumers of `quoin-core`.
//!
//! Covers the core traits, the blanket extension traits, the reactive-effect
//! runtime, and the task helpers, so one import line unlocks the full
//! framework-agnostic API:
//!
//! ```text
//! use quoin_core::prelude::*;
//!
//! let cx = /* any adapter context */;
//! let count = cx.create_signal(0u32);
//! let label = count.map(|n| format!("count = {n}"));   // SignalExt
//! let res = cx.create_resource(fetch);                  // ReactiveContextExt
//! let (handle, token) = cx.executor().spawn_with_token(|t| async move { … });
//! let _effect = watch(&count, |n| println!("count → {n}")); // effect runtime
//! ```

pub use crate::batch::BatchGuard;
pub use crate::local::{pending_local_tasks, run_local_tasks};
pub use crate::{
    AbortJoinHandle, AbortTask, AnySignal, CancelGuard, CancellationToken, Cancelled,
    CancelledFuture, DepHandle, Derived, EffectHandle, EffectScope, Executor, ExecutorExt,
    JoinHandle, LocalExecutor, LocalJoinHandle, LocalTask, Memo, NotifierSignal, RawTask,
    ReactiveContext, ReactiveContextExt, Resource, ResourceState, RunUntilCancelled,
    SharedJoinHandle, Signal, SignalExt, SleepFuture, SyncSleepFuture, TrackedSignal, Zipped,
    create_effect, derived, memo, sleep, spawn_abortable, spawn_local_task, watch, zip,
};
