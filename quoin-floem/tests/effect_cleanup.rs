//! `effect!` cleanup semantics on the Floem backend.
//!
//! The `effect!` expansion installs the user's one cleanup closure in a
//! shared [`CleanupSlot`]: the effect runs it before every body re-run, and
//! a slot clone owned by a scope-bound `RwSignal` runs it one final time
//! when the owning reactive scope is disposed. These tests exercise both
//! halves against the real `floem_reactive` runtime.

use std::cell::RefCell;
use std::rc::Rc;

use floem::reactive::{Effect, Runtime, RwSignal, Scope, SignalTrack, SignalUpdate};
use quoin_floem::effect::CleanupSlot;

/// Drives the runtime on the test thread (floem's reactive runtime is
/// UI-thread-only; tests establish the thread the same way its own tests
/// do).
fn with_ui_runtime<T>(f: impl FnOnce() -> T) -> T {
    Runtime::init_on_ui_thread();
    f()
}

#[test]
fn cleanup_runs_before_each_rerun() {
    with_ui_runtime(|| {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));

        let source = RwSignal::new(0i32);

        // The shape the `effect! { deps: [..], run: .., cleanup: .. }`
        // expansion emits: the cleanup closure is installed once, outside
        // the effect.
        let slot = CleanupSlot::new();
        {
            let log = Rc::clone(&log);
            slot.set(Box::new(move || log.borrow_mut().push("cleanup")));
        }
        RwSignal::new(slot.clone());
        Effect::new({
            let log = Rc::clone(&log);
            move |prev: Option<()>| {
                if prev.is_some() {
                    slot.run();
                }
                // Track the source the way a real effect body does (a quoin
                // `Signal::get` tracks) so the re-run actually re-runs.
                source.track();
                log.borrow_mut().push("run");
            }
        });

        // Initial run: no previous state, so no cleanup yet.
        assert_eq!(*log.borrow(), vec!["run"]);

        // Re-run: cleanup first, then the body.
        source.set(1);
        Runtime::drain_pending_work();
        assert_eq!(*log.borrow(), vec!["run", "cleanup", "run"]);

        source.set(2);
        Runtime::drain_pending_work();
        assert_eq!(
            *log.borrow(),
            vec!["run", "cleanup", "run", "cleanup", "run"]
        );
    });
}

#[test]
fn cleanup_runs_on_scope_disposal() {
    with_ui_runtime(|| {
        let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));

        {
            let scope = Scope::new();
            scope.enter(|| {
                // Exactly the owners the `effect!` expansion creates: the
                // scope-bound signal and the effect closure. The setup
                // handle drops at the end of this closure, so disposing the
                // scope drops the last two handles — which runs the
                // cleanup, the "drop" half of the contract.
                let slot = CleanupSlot::new();
                {
                    let log = Rc::clone(&log);
                    slot.set(Box::new(move || log.borrow_mut().push("cleanup")));
                }
                RwSignal::new(slot.clone());
                let inner = Rc::clone(&log);
                Effect::new({
                    move |prev: Option<()>| {
                        if prev.is_some() {
                            slot.run();
                        }
                        inner.borrow_mut().push("run");
                    }
                });
            });

            // Body ran once inside the scope; no re-run happened, so no
            // pre-run cleanup has fired yet.
            assert_eq!(*log.borrow(), vec!["run"]);

            scope.dispose();
            assert_eq!(*log.borrow(), vec!["run", "cleanup"]);
        }
    });
}

#[test]
fn sibling_slot_drops_do_not_run_the_cleanup() {
    with_ui_runtime(|| {
        // Clones share the slot: only the LAST handle drop (the scope-owned
        // signal) runs the closure, so mere clone juggling never fires it.
        let observed = Rc::new(RefCell::new(0));
        let slot = CleanupSlot::new();
        {
            let observed = Rc::clone(&observed);
            slot.set(Box::new(move || *observed.borrow_mut() += 1));
        }
        let sibling = slot.clone();
        drop(sibling);
        assert_eq!(*observed.borrow(), 0);
        drop(slot); // last handle
        assert_eq!(*observed.borrow(), 1);
        // Re-running after the drop finds the slot consumed: no double run.
        let revived = CleanupSlot::new();
        drop(revived);
        assert_eq!(*observed.borrow(), 1);
    });
}
