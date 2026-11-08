//! Compile-time pin: one `use quoin_core::prelude::*` must unlock the full
//! framework-agnostic API the crate docs promise — the effect runtime
//! (`create_effect`, `watch`, `EffectHandle`, `EffectScope`, `DepHandle`,
//! `TrackedSignal`), the erasure layer (`AnySignal`, `NotifierSignal`), the
//! computed layer (`zip`/`Zipped`), batching (`BatchGuard`), and the task
//! API (`spawn_abortable`, `spawn_local_task`, their handles).
//!
//! Every name below is referenced in a typed position so a missing prelude
//! export fails compilation, not just clippy.

use quoin_core::prelude::*;

fn takes_tracked<T: TrackedSignal<u32> + Send + Sync + 'static>(_signal: &T) {}

#[test]
fn prelude_unlocks_the_effect_and_task_api() {
    // Effect runtime.
    let source = quoin_core::test_support::static_signal(0u32);
    let erased: AnySignal<u32> = AnySignal::new_tracked(&source);
    let handle: EffectHandle = watch(&erased, |_value: &u32| {});
    let scope = EffectScope::new();
    scope.create_effect(|| {});
    let dep: DepHandle = DepHandle::new();
    assert!(!dep.ptr_eq(&source.dep_handle()));

    takes_tracked(&source);
    takes_tracked(&erased);

    handle.dispose();
    assert!(handle.is_disposed());
    scope.dispose_all();

    // Erasure + notifier cells.
    let notifier: NotifierSignal<u32> = NotifierSignal::new(1u32, || {});
    notifier.set(2);
    assert_eq!(notifier.get(), 2);

    // Computed layer.
    let a = quoin_core::test_support::static_signal(1u32);
    let b = quoin_core::test_support::static_signal(2u32);
    let pair: Zipped<_, _, u32, u32> = zip(&a, &b);
    assert_eq!(pair.get(), (1, 2));

    // Batching.
    let guard = BatchGuard::enter();
    assert!(quoin_core::batch::in_batch());
    drop(guard);
    assert!(!quoin_core::batch::in_batch());

    // Task API: abortable spawn + local spawn.
    let task: AbortJoinHandle<u8> = spawn_abortable(async { 1u8 });
    task.abort();
    let local: LocalTask<u8> = spawn_local_task(async { 2u8 });
    local.abort();
    // An aborted local task is still drained from the thread-local queue
    // (abort stops the body, not the drain) — see local.rs's own tests.
    assert_eq!(quoin_core::local::run_local_tasks(), 1);
}
