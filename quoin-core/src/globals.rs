//! Thread-local, type-keyed global storage shared by the adapters.
//!
//! The pull-based adapters (GPUI, Xilem, Floem) implement
//! `provide_global`/`use_global` with a thread-local `TypeId`-keyed map:
//! globals are visible on the thread that registered them, matching each
//! framework's main-thread assumptions. That map — thread-local declaration,
//! insertion, downcast lookup — used to be hand-copied in all three adapter
//! crates. It now lives here once.
//!
//! Keys are `(TypeId of the context type, TypeId of the stored value)`, so
//! two adapters running on the same thread (each with their own signal
//! wrapper type) never collide, and one context can hold one global per
//! value type.
//!
//! # Example
//!
//! ```
//! use quoin_core::globals;
//!
//! struct MyApp;
//! struct MySignal(u32);
//! impl Clone for MySignal {
//!     fn clone(&self) -> Self {
//!         Self(self.0)
//!     }
//! }
//!
//! globals::insert::<MyApp, MySignal>(MySignal(7));
//! let signal = globals::get::<MyApp, MySignal>().expect("just inserted");
//! assert_eq!(signal.0, 7);
//! ```

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static GLOBALS: RefCell<HashMap<(TypeId, TypeId), Box<dyn Any + Send + Sync>>> =
        RefCell::new(HashMap::new());
}

/// Stores `value` under `(C, V)` for the current thread, replacing any
/// previous value of the same `(context, value)` pair.
pub fn insert<C: 'static, V: Any + Send + Sync>(value: V) {
    GLOBALS.with(|store| {
        store
            .borrow_mut()
            .insert((TypeId::of::<C>(), TypeId::of::<V>()), Box::new(value));
    });
}

/// Returns a clone of the value stored under `(C, V)`, if this thread has
/// one. The value type must be `Clone` because every consumer shares the
/// same stored cell through its own handle.
#[must_use]
pub fn get<C: 'static, V: Any + Send + Sync + Clone>() -> Option<V> {
    GLOBALS.with(|store| {
        store
            .borrow()
            .get(&(TypeId::of::<C>(), TypeId::of::<V>()))
            .and_then(|boxed| boxed.downcast_ref::<V>())
            .cloned()
    })
}

/// Removes the value stored under `(C, V)`; returns `true` when one existed.
#[must_use = "the return value reports whether a value was removed"]
pub fn remove<C: 'static, V: Any + Send + Sync>() -> bool {
    GLOBALS.with(|store| {
        store
            .borrow_mut()
            .remove(&(TypeId::of::<C>(), TypeId::of::<V>()))
            .is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ContextA;
    struct ContextB;

    #[derive(Clone)]
    struct Wrapper(u32);

    #[test]
    fn insert_then_get_round_trips() {
        insert::<ContextA, Wrapper>(Wrapper(11));
        assert_eq!(get::<ContextA, Wrapper>().map(|w| w.0), Some(11));
    }

    #[test]
    fn different_context_types_do_not_collide() {
        insert::<ContextA, Wrapper>(Wrapper(1));
        insert::<ContextB, Wrapper>(Wrapper(2));
        assert_eq!(get::<ContextA, Wrapper>().map(|w| w.0), Some(1));
        assert_eq!(get::<ContextB, Wrapper>().map(|w| w.0), Some(2));
    }

    #[test]
    fn get_missing_key_is_none() {
        #[derive(Clone)]
        struct NeverStored;
        assert!(get::<ContextA, NeverStored>().is_none());
    }

    #[test]
    fn insert_replaces_previous_value() {
        insert::<ContextA, Wrapper>(Wrapper(1));
        insert::<ContextA, Wrapper>(Wrapper(9));
        assert_eq!(get::<ContextA, Wrapper>().map(|w| w.0), Some(9));
    }

    #[test]
    fn remove_clears_and_reports() {
        insert::<ContextA, Wrapper>(Wrapper(3));
        assert!(remove::<ContextA, Wrapper>());
        assert!(get::<ContextA, Wrapper>().is_none());
        assert!(!remove::<ContextA, Wrapper>());
    }

    #[test]
    fn values_are_thread_local() {
        insert::<ContextA, Wrapper>(Wrapper(5));
        let seen = std::thread::spawn(|| get::<ContextA, Wrapper>().map(|w| w.0))
            .join()
            .expect("join");
        assert_eq!(seen, None, "globals are per-thread by contract");
        assert_eq!(get::<ContextA, Wrapper>().map(|w| w.0), Some(5));
    }
}
