//! End-to-end user-flow tests for the todo store.
//!
//! These run the *complete* application logic — the same code a view layer
//! would drive — against `quoin-testkit`'s deterministic context. They are
//! the quoin equivalent of "click through the whole app" e2e suites, without
//! a browser or window manager: every scenario below is what a real user
//! session looks like at the state level.

use quoin_core::prelude::*;
use quoin_testkit::{ImmediateExecutor, TestContext};
use todo_core::{Filter, TodoStore};

#[test]
fn e2e_full_user_session() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let store = TodoStore::new(&cx);

    // 1. The user types three items, pressing Enter after each.
    store.set_draft("buy milk".into());
    store.commit_draft();
    store.set_draft("walk dog".into());
    store.commit_draft();
    store.set_draft("write quoin docs".into());
    store.commit_draft();
    assert_eq!(store.visible().len(), 3);
    assert_eq!(store.footer_text().get(), "3 items left");

    // 2. Completes the first two.
    assert!(store.toggle(1));
    assert!(store.toggle(2));
    assert_eq!(store.footer_text().get(), "1 item left");
    assert!(!store.all_completed());

    // 3. Switches to the Active filter: only the third shows.
    store.set_filter(Filter::Active);
    assert_eq!(
        store
            .visible()
            .iter()
            .map(|t| t.title.as_str())
            .collect::<Vec<_>>(),
        vec!["write quoin docs"]
    );

    // 4. Completes the last one from the Active tab — the list empties.
    assert!(store.toggle(3));
    assert!(store.visible().is_empty());
    assert_eq!(store.footer_text().get(), "All done!");
    assert!(store.all_completed());

    // 5. Clear-completed wipes everything.
    store.set_filter(Filter::All);
    store.clear_completed();
    assert!(store.visible().is_empty());
}

#[test]
fn e2e_no_op_interactions_produce_zero_renders() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let store = TodoStore::new(&cx);
    store.add("a");
    store.add("b");

    cx.updates().reset();

    // Toggling all when nothing is completed is a no-op:
    store.set_all_completed(false);
    // Setting the same filter twice:
    store.set_filter(Filter::All);
    store.set_filter(Filter::All);
    // Writing the same draft text:
    store.set_draft(store.draft.get());
    // Committing an empty draft:
    store.set_draft("".into());
    store.commit_draft();

    assert_eq!(
        cx.updates().count(),
        0,
        "no-op interactions must not request re-renders"
    );
}

#[test]
fn e2e_commit_is_one_render_transaction() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let store = TodoStore::new(&cx);

    cx.updates().reset();
    store.set_draft("one write".into());
    let after_draft = cx.updates().count();

    store.commit_draft();
    // commit mutates next_id + todos + draft inside one batch => exactly one
    // additional update request on pull-based adapters.
    assert_eq!(
        cx.updates().count(),
        after_draft + 1,
        "batched commit must collapse into a single update request"
    );
}

#[test]
fn e2e_remove_unknown_item_is_safe() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let store = TodoStore::new(&cx);
    store.add("only");
    assert!(!store.remove(404));
    assert!(!store.toggle(404));
    assert_eq!(store.visible().len(), 1);
}

#[test]
fn e2e_state_survives_context_clone() {
    // Signals are Clone-shared; a view struct cloning the context (or the
    // signals) must observe the same data.
    let cx = TestContext::with_executor(ImmediateExecutor);
    let store = TodoStore::new(&cx);
    store.add("shared");
    let cloned = store.clone_via_signals();
    let items = cloned.visible();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "shared");
}

#[test]
fn e2e_threaded_executor_stress() {
    // The same flows on the ThreadedExecutor prove the store has no hidden
    // single-thread assumptions in its value semantics.
    let cx = TestContext::new();
    let store = TodoStore::new(&cx);
    for i in 0..100 {
        store.add(&format!("item {i}"));
    }
    assert_eq!(store.visible().len(), 100);
    for id in 1..=100 {
        assert!(store.toggle(id));
    }
    assert_eq!(store.active_count(), 0);
    store.clear_completed();
    assert!(store.visible().is_empty());
}
