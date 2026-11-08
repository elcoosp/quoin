//! Component-scoped effects for the Xilem backend, running on the core
//! reactive runtime.
//!
//! Xilem itself has no reactive graph — the `effect!` lowering targets
//! [`quoin_core::create_effect`] instead, which observes every
//! [`XilemSignal`](crate::XilemSignal) through its dependency cell (the
//! signal type is [`quoin_core::NotifierSignal`], a [`TrackedSignal`]).
//! This module adds the two pieces the emitter needs on top:
//!
//! - [`CleanupSlot`] — the `Send + Sync` counterpart of
//!   `quoin::floem_effect::CleanupSlot`: the user's `cleanup:` closure runs
//!   before every effect re-run, and a final time when the effect node is
//!   dropped (scope disposal or explicit dispose).
//! - The **component effect scope**: `component!` on Xilem generates a state
//!   struct whose `new(ctx)` runs exactly once. While `on_mount` executes,
//!   the generated code holds this module's current-scope guard, so every
//!   `effect!` invoked there registers into a
//!   [`quoin_core::EffectScope`] owned by the component struct. Dropping the
//!   component disposes its effects — the Xilem counterpart of floem's
//!   scope-bound disposal. Outside a component mount (standalone use,
//!   tests), [`create_effect`] falls back to a detached core effect that
//!   runs until explicitly disposed.
//!
//! # Placement contract
//!
//! Effects must be created at component scope (`on_mount { .. }`), never in
//! the render body: `app_logic` re-runs on every rebuild, so an effect
//! created there would multiply per rebuild. The `component!` emitter
//! rejects `effect!` in the render block with a named diagnostic rather
//! than silently leaking (see `docs/BACKEND-PARITY.md`).
//!
//! # Threading
//!
//! The current-scope registry is thread-local: component construction (and
//! therefore `on_mount`) happens on one thread, and every effect created
//! during that window lands in the component scope. Effects created on other
//! threads are detached — the same behaviour as no scope being active.

use quoin_core::{EffectHandle, EffectScope};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// A repeatable effect cleanup: the slot holds the one closure the user
/// registered and runs it on demand, as many times as the effect re-runs.
///
/// This is the `Send + Sync` counterpart of `quoin::floem_effect::CleanupSlot`
/// (there `Rc`/`RefCell`, here `Arc`/`Mutex`, because core effect closures
/// are `Send`). Drop semantics match: the cleanup runs when the **last**
/// handle drops, so the clone living inside the effect body fires it when
/// the effect node is disposed and dropped — the "dispose" half of the
/// floem contract, where the final run is driven by the component effect
/// scope being dropped with the component.
#[derive(Clone)]
pub struct CleanupSlot {
    inner: SlotInner,
}

/// The shared slot behind a [`CleanupSlot`] (factored out for readability).
type SlotInner = Arc<Mutex<Option<CleanupFn>>>;

/// The installed cleanup closure.
type CleanupFn = Box<dyn Fn() + Send + 'static>;

impl Default for CleanupSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupSlot {
    /// Creates an empty slot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Installs the cleanup closure (the `effect!` expansion calls this
    /// once, outside the effect, so the closure never has to move after).
    ///
    /// # Panics
    /// Panics if the slot mutex is poisoned (a prior panic while holding it).
    pub fn set(&self, cleanup: CleanupFn) {
        *self.inner.lock().unwrap() = Some(cleanup);
    }

    /// Runs the cleanup if one is installed. The closure stays installed —
    /// the next re-run (or the final drop) runs it again.
    ///
    /// # Panics
    /// Panics if the slot mutex is poisoned (a prior panic while holding it).
    pub fn run(&self) {
        if let Some(cleanup) = self.inner.lock().unwrap().as_ref() {
            cleanup();
        }
    }
}

impl Drop for CleanupSlot {
    fn drop(&mut self) {
        // Arc semantics: the closure runs when the LAST handle drops (the
        // clone inside the disposed effect body), and earlier drops of
        // sibling clones find `Arc::strong_count > 1` and no-op.
        if Arc::strong_count(&self.inner) == 1
            && let Ok(mut guard) = self.inner.lock()
            && let Some(cleanup) = guard.take()
        {
            cleanup();
        }
    }
}

// ---------------------------------------------------------------------------
// Component effect scope
// ---------------------------------------------------------------------------

thread_local! {
    /// The stack of component effect scopes currently being constructed.
    /// Nesting mirrors nested component construction; the top of the stack
    /// receives every effect created through [`create_effect`].
    static SCOPES: RefCell<Vec<Arc<EffectScope>>> = const { RefCell::new(Vec::new()) };
}

/// Owns one entry on the current-scope stack; dropping it pops.
///
/// The `component!` expansion holds this guard for the duration of
/// `on_mount`, so every `effect!` invoked there registers into the
/// component's [`EffectScope`].
#[must_use = "the scope is only active while the guard is alive"]
pub struct ScopeGuard {
    active: bool,
}

impl ScopeGuard {
    /// Pops the scope early (the `component!` expansion calls this once
    /// `on_mount` is done; `Drop` covers the panic path).
    pub fn pop(mut self) {
        self.active = false;
        SCOPES.with(|scopes| {
            scopes.borrow_mut().pop();
        });
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        if self.active {
            self.active = false;
            SCOPES.with(|scopes| {
                scopes.borrow_mut().pop();
            });
        }
    }
}

/// Marks the current thread as constructing a component: every
/// [`create_effect`] call until the returned guard is dropped (or
/// [`ScopeGuard::pop`] is called) registers into `scope`.
pub fn enter_effect_scope(scope: Arc<EffectScope>) -> ScopeGuard {
    SCOPES.with(|scopes| {
        scopes.borrow_mut().push(scope);
    });
    ScopeGuard { active: true }
}

/// The component scope currently being constructed on this thread, if any.
#[must_use]
pub fn current_scope() -> Option<Arc<EffectScope>> {
    SCOPES.with(|scopes| scopes.borrow().last().cloned())
}

/// Creates an effect the way the `effect!` Xilem expansion does: into the
/// component scope currently being constructed (see
/// [`enter_effect_scope`]), or detached when none is active.
///
/// The reactivity itself is [`quoin_core::create_effect`]: the body runs
/// once immediately, then re-runs whenever a [`XilemSignal`](crate::XilemSignal)
/// it read on the previous run is written — batch-coalesced, FIFO, with
/// deferred self-writes. Disposal happens when the owning component scope
/// drops (component teardown) or through the returned handle.
pub fn create_effect<F>(run: F) -> EffectHandle
where
    F: FnMut() + Send + 'static,
{
    match current_scope() {
        Some(scope) => scope.create_effect(run),
        None => quoin_core::create_effect(run),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::XilemContext;
    use quoin_core::{ReactiveContext, Signal};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn counter() -> Arc<AtomicUsize> {
        Arc::new(AtomicUsize::new(0))
    }

    fn runs(c: &Arc<AtomicUsize>) -> usize {
        c.load(Ordering::SeqCst)
    }

    fn test_context() -> XilemContext {
        let runtime =
            Arc::new(xilem::tokio::runtime::Runtime::new().expect("tokio runtime for tests"));
        XilemContext::new(runtime)
    }

    // -- CleanupSlot --------------------------------------------------------

    #[test]
    fn cleanup_slot_run_is_repeatable() {
        let ran = counter();
        let slot = CleanupSlot::new();
        let ran_cb = Arc::clone(&ran);
        slot.set(Box::new(move || {
            ran_cb.fetch_add(1, Ordering::SeqCst);
        }));
        slot.run();
        slot.run();
        assert_eq!(runs(&ran), 2, "the same closure runs per re-run");
    }

    #[test]
    fn cleanup_slot_runs_on_last_handle_drop() {
        let ran = counter();
        let slot = CleanupSlot::new();
        let ran_cb = Arc::clone(&ran);
        slot.set(Box::new(move || {
            ran_cb.fetch_add(1, Ordering::SeqCst);
        }));
        let clone = slot.clone();
        drop(clone); // shared slot: no run yet
        assert_eq!(runs(&ran), 0);
        drop(slot); // last handle: the final cleanup
        assert_eq!(runs(&ran), 1, "dispose drops the body's clone and runs");
    }

    #[test]
    fn cleanup_slot_without_closure_drops_silently() {
        let slot = CleanupSlot::new();
        let clone = slot.clone();
        drop(clone);
        drop(slot); // nothing installed: nothing to run
    }

    #[test]
    fn cleanup_slot_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CleanupSlot>();
    }

    // -- Scope registry ------------------------------------------------------

    #[test]
    fn scope_guard_is_current_until_popped() {
        assert!(current_scope().is_none(), "no scope outside a component");
        let scope = Arc::new(EffectScope::new());
        let guard = enter_effect_scope(Arc::clone(&scope));
        assert!(
            current_scope().is_some_and(|s| Arc::ptr_eq(&s, &scope)),
            "the entered scope is the current one"
        );
        guard.pop();
        assert!(current_scope().is_none(), "pop removes the scope");
    }

    #[test]
    fn nested_scopes_shadow_and_restore() {
        let outer = Arc::new(EffectScope::new());
        let inner = Arc::new(EffectScope::new());
        let outer_guard = enter_effect_scope(Arc::clone(&outer));
        let inner_guard = enter_effect_scope(Arc::clone(&inner));
        assert!(
            current_scope().is_some_and(|s| Arc::ptr_eq(&s, &inner)),
            "the innermost scope wins"
        );
        inner_guard.pop();
        assert!(
            current_scope().is_some_and(|s| Arc::ptr_eq(&s, &outer)),
            "the outer scope is restored"
        );
        outer_guard.pop();
        assert!(current_scope().is_none());
    }

    #[test]
    fn scope_guard_drop_pops_without_explicit_pop() {
        let scope = Arc::new(EffectScope::new());
        {
            let _guard = enter_effect_scope(Arc::clone(&scope));
            assert!(current_scope().is_some());
        }
        assert!(
            current_scope().is_none(),
            "Drop pops the scope (panic path)"
        );
    }

    // -- Effect routing ------------------------------------------------------

    #[test]
    fn effects_inside_a_component_scope_dispose_with_the_scope() {
        let cx = test_context();
        let source = cx.create_signal(0u32);
        let runs_count = counter();

        let scope = Arc::new(EffectScope::new());
        let (source2, runs2) = (source.clone(), Arc::clone(&runs_count));
        {
            let guard = enter_effect_scope(Arc::clone(&scope));
            let _handle = create_effect(move || {
                let _ = source2.get();
                runs2.fetch_add(1, Ordering::SeqCst);
            });
            guard.pop();
        }
        assert_eq!(runs(&runs_count), 1, "initial run at creation");
        source.set(1);
        assert_eq!(runs(&runs_count), 2, "scoped effect still re-runs");

        // Component teardown: dropping the scope disposes its effects.
        drop(scope);
        source.set(2);
        assert_eq!(runs(&runs_count), 2, "disposed effects never re-run");
    }

    #[test]
    fn effects_outside_a_scope_are_detached() {
        let cx = test_context();
        let source = cx.create_signal(0u32);
        let runs_count = counter();

        let (source2, runs2) = (source.clone(), Arc::clone(&runs_count));
        let _handle = create_effect(move || {
            let _ = source2.get();
            runs2.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs(&runs_count), 1);
        source.set(1);
        assert_eq!(runs(&runs_count), 2);
        source.set(2);
        assert_eq!(runs(&runs_count), 3, "detached effects keep running");
    }

    #[test]
    fn disposed_scoped_effect_runs_its_final_cleanup() {
        let cx = test_context();
        let source = cx.create_signal(0u32);
        let cleanups = counter();

        let scope = Arc::new(EffectScope::new());
        let slot = CleanupSlot::new();
        let cleanups_cb = Arc::clone(&cleanups);
        slot.set(Box::new(move || {
            cleanups_cb.fetch_add(1, Ordering::SeqCst);
        }));

        let (source2, slot2) = (source.clone(), slot.clone());
        {
            let guard = enter_effect_scope(Arc::clone(&scope));
            // The emitted `effect!` shape: the first run has no previous
            // state to tear down, so the cleanup only arms from run two on.
            let mut first_run = true;
            let _handle = create_effect(move || {
                if first_run {
                    first_run = false;
                } else {
                    slot2.run();
                }
                let _ = source2.get();
            });
            guard.pop();
        }
        drop(slot); // the effect body owns the last handle now
        assert_eq!(runs(&cleanups), 0, "no cleanup before any re-run");

        source.set(1);
        assert_eq!(runs(&cleanups), 1, "re-run cleans up the previous state");

        // Component teardown: the disposed body drops, firing the final
        // cleanup through the slot's last-handle drop.
        drop(scope);
        assert_eq!(runs(&cleanups), 2, "final cleanup on disposal");
        source.set(2); // no panic: the effect is gone
    }
}
