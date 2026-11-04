//! Effect cleanup for the `effect!` Floem lowering.
//!
//! Floem's reactive runtime has no `on_cleanup` hook: an `Effect` is dropped
//! silently when its owning [`Scope`](floem_reactive::Scope) is disposed,
//! and its `Effect::new` constructor returns no handle. The `effect!` macro
//! therefore stores the user's cleanup closure in a shared [`CleanupSlot`]:
//!
//! - the effect calls `run()` before every body re-run, so the previous
//!   state is torn down first (the "re-run" half);
//! - one [`CleanupSlot`] clone lives in a scope-owned `RwSignal`, so scope
//!   disposal drops the signal, which drops the guard, which runs the
//!   cleanup one final time (the "drop" half).
//!
//! The cleanup must be `Fn` — it is the one closure the user wrote for the
//! effect, and it has to survive being run before every re-run. Everything
//! here is UI-thread-only (`Rc`/`RefCell`), matching floem's
//! `Runtime::assert_ui_thread` contract.

use std::cell::RefCell;
use std::rc::Rc;

/// A repeatable effect cleanup: the slot holds the one closure the user
/// registered and runs it on demand, as many times as the effect re-runs.
#[derive(Clone)]
pub struct CleanupSlot {
    inner: SlotInner,
}

/// The shared slot behind a [`CleanupSlot`] (factored out for readability).
type SlotInner = Rc<RefCell<Option<CleanupFn>>>;

/// The installed cleanup closure.
type CleanupFn = Box<dyn Fn() + 'static>;

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
            inner: Rc::new(RefCell::new(None)),
        }
    }

    /// Installs the cleanup closure (the `effect!` expansion calls this
    /// once, outside the effect, so the closure never has to move after).
    pub fn set(&self, cleanup: CleanupFn) {
        *self.inner.borrow_mut() = Some(cleanup);
    }

    /// Runs the cleanup if one is installed. The closure stays installed —
    /// the next re-run (or the final drop) runs it again.
    pub fn run(&self) {
        if let Some(cleanup) = self.inner.borrow().as_ref() {
            cleanup();
        }
    }
}

impl Drop for CleanupSlot {
    fn drop(&mut self) {
        // Rc semantics: the closure runs when the LAST handle drops (the
        // scope-owned signal in the `effect!` expansion), and earlier drops
        // of sibling clones find `Rc::strong_count > 1` and no-op.
        if Rc::strong_count(&self.inner) == 1
            && let Some(cleanup) = self.inner.borrow_mut().take()
        {
            cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn run_is_repeatable() {
        let ran = Rc::new(RefCell::new(0));
        let slot = CleanupSlot::new();
        let ran_cb = Rc::clone(&ran);
        slot.set(Box::new(move || *ran_cb.borrow_mut() += 1));
        slot.run();
        slot.run();
        assert_eq!(*ran.borrow(), 2);
    }

    #[test]
    fn drop_runs_once_for_the_last_handle() {
        let ran = Rc::new(RefCell::new(0));
        let slot = CleanupSlot::new();
        let ran_cb = Rc::clone(&ran);
        slot.set(Box::new(move || *ran_cb.borrow_mut() += 1));
        let clone = slot.clone();
        drop(clone); // shared slot: no run yet
        assert_eq!(*ran.borrow(), 0);
        drop(slot); // last handle: runs the cleanup
        assert_eq!(*ran.borrow(), 1);
    }

    #[test]
    fn empty_slot_drops_silently() {
        let slot = CleanupSlot::new();
        let clone = slot.clone();
        drop(clone);
        drop(slot); // no cleanup installed: nothing to run
    }
}
