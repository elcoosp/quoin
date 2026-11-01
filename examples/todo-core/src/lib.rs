//! Framework-agnostic todo application store.
//!
//! A complete "app brain" written once against [`quoin_core`]'s traits: no
//! UI framework, no threads, no I/O. The same store can be wired into GPUI,
//! Leptos, Dioxus, Xilem, or Floem views — and, crucially, it is fully e2e
//! testable headlessly (see `tests/e2e.rs`).
//!
//! The store demonstrates the idioms the whole API surface was designed
//! around:
//!
//! - [`ReactiveContext::create_signal`] for owned state,
//! - [`SignalExt::set_eq`] to avoid redundant renders on no-op writes,
//! - [`ReactiveContextExt::batch`] for multi-signal transactions,
//! - [`derived`] for computed views (filtered lists, counters),
//! - [`CancellationToken::child_token`] for per-item async work.

use quoin_core::prelude::*;
use std::rc::Rc;

/// Visibility filter for the todo list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Filter {
    /// Show everything.
    #[default]
    All,
    /// Show only incomplete items.
    Active,
    /// Show only completed items.
    Completed,
}

impl Filter {
    /// Machine-friendly name (used by view layers as tab ids).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }
}

/// One todo item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    /// Unique, monotonically increasing identifier.
    pub id: u64,
    /// User-facing text.
    pub title: String,
    /// Whether the item is checked off.
    pub completed: bool,
}

/// The complete state of the todo app.
///
/// Generic over the signal type `S`, so it works with every adapter (and the
/// testkit): `TodoStore<C::Signal<Vec<Todo>>, ...>`.
pub struct TodoStore<C: ReactiveContext> {
    /// The context is kept so multi-signal mutations can run in one
    /// [`quoin_core::batch`] (one re-render request instead of four).
    pub cx: C,
    /// All items, oldest first.
    pub todos: C::Signal<Vec<Todo>>,
    /// Current visibility filter.
    pub filter: C::Signal<Filter>,
    /// Contents of the "new item" input box.
    pub draft: C::Signal<String>,
    /// Monotonic id source.
    next_id: C::Signal<u64>,
}

impl<C: ReactiveContext> TodoStore<C> {
    /// Creates an empty store.
    #[must_use]
    pub fn new(cx: &C) -> Self {
        Self {
            cx: cx.clone(),
            todos: cx.create_signal(Vec::new()),
            filter: cx.create_signal(Filter::All),
            draft: cx.create_signal(String::new()),
            next_id: cx.create_signal(0u64),
        }
    }

    /// Commits the current draft as a new todo (no-op for blank drafts).
    ///
    /// All three signal writes happen inside one [`batch`], so pull-based
    /// view layers re-render exactly once.
    pub fn commit_draft(&self) {
        let title = self.draft.get().trim().to_string();
        if title.is_empty() {
            return;
        }
        self.batch_commit(title);
    }

    fn batch_commit(&self, title: String) {
        // One logical transaction: id bump + push + draft reset produce a
        // single re-render request on pull-based adapters.
        self.cx.batch(|| {
            self.next_id.update(|id| *id += 1);
            let id = self.next_id.get();
            self.todos.update(|list| {
                list.push(Todo {
                    id,
                    title,
                    completed: false,
                });
            });
            self.draft.set(String::new());
        });
    }

    /// Adds a todo programmatically (used by tests and seeding).
    pub fn add(&self, title: &str) {
        self.draft.set(title.to_string());
        self.commit_draft();
    }

    /// Toggles an item's completed flag; returns `false` if `id` is unknown.
    pub fn toggle(&self, id: u64) -> bool {
        let mut found = false;
        self.todos.update(|list| {
            for todo in list.iter_mut() {
                if todo.id == id {
                    todo.completed = !todo.completed;
                    found = true;
                    break;
                }
            }
        });
        found
    }

    /// Removes an item; returns `true` when something was removed.
    pub fn remove(&self, id: u64) -> bool {
        let mut removed = false;
        self.todos.update(|list| {
            let before = list.len();
            list.retain(|todo| todo.id != id);
            removed = list.len() != before;
        });
        removed
    }

    /// Removes all completed items.
    pub fn clear_completed(&self) {
        self.todos
            .update(|list| list.retain(|todo| !todo.completed));
    }

    /// Marks every item completed (or clears all flags when `completed` is
    /// false). Skips the write entirely when the state already matches —
    /// the "toggle all" checkbox in a view layer stays stable.
    pub fn set_all_completed(&self, completed: bool) {
        let needs_change = self
            .todos
            .with(|list| list.iter().any(|t| t.completed != completed));
        if !needs_change {
            return;
        }
        self.todos.update(|list| {
            for todo in list.iter_mut() {
                todo.completed = completed;
            }
        });
    }

    /// Sets the visibility filter. Returns `true` when it changed.
    pub fn set_filter(&self, filter: Filter) -> bool {
        self.filter.set_eq(filter)
    }

    /// Updates the draft text (no-op write when unchanged — keeps text
    /// inputs from re-rendering on every keystroke that changes nothing).
    pub fn set_draft(&self, text: String) {
        self.draft.set_eq(text);
    }

    /// The items visible under the current filter (computed view).
    #[must_use]
    pub fn visible(&self) -> Vec<Todo> {
        let filter = self.filter.get();
        self.todos.with(|list| {
            list.iter()
                .filter(|todo| match filter {
                    Filter::All => true,
                    Filter::Active => !todo.completed,
                    Filter::Completed => todo.completed,
                })
                .cloned()
                .collect()
        })
    }

    /// Number of incomplete items (the classic footer counter).
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.todos
            .with(|list| list.iter().filter(|t| !t.completed).count())
    }

    /// Whether the "toggle all" checkbox should be checked.
    #[must_use]
    pub fn all_completed(&self) -> bool {
        self.todos
            .with(|list| !list.is_empty() && list.iter().all(|t| t.completed))
    }

    /// Returns an independent store handle sharing the same underlying
    /// signals (view layers clone signals across components).
    #[must_use]
    pub fn clone_via_signals(&self) -> Self {
        Self {
            cx: self.cx.clone(),
            todos: self.todos.clone(),
            filter: self.filter.clone(),
            draft: self.draft.clone(),
            next_id: self.next_id.clone(),
        }
    }

    /// A derived signal producing the footer summary line, e.g. `"2 items
    /// left"`. Recomputed on read; view layers can bind it directly.
    #[must_use]
    pub fn footer_text(&self) -> impl Signal<String> {
        let todos = self.todos.clone();
        derived(&todos, |list: &Vec<Todo>| {
            let active = list.iter().filter(|t| !t.completed).count();
            match active {
                0 => "All done!".to_string(),
                1 => "1 item left".to_string(),
                n => format!("{n} items left"),
            }
        })
    }
}

/// A spawned "item saved" indicator driven by a cancellation token — shows
/// how per-item async work is scoped in quoin apps.
pub struct SaveIndicator {
    /// Cancels the indicator's work when the owning scope goes away.
    pub token: CancellationToken,
}

impl SaveIndicator {
    /// Creates an indicator whose lifetime is bound to the given context's
    /// token (usually the app root): cancelling the parent cancels every
    /// per-item child.
    #[must_use]
    pub fn new(parent: &CancellationToken) -> Self {
        Self {
            token: parent.child_token(),
        }
    }
}

/// Convenience alias for handlers: framework view layers store these in
/// their own structs.
pub type SharedHandler = Rc<dyn Fn()>;

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_testkit::{ImmediateExecutor, TestContext};

    #[test]
    fn store_starts_empty() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        assert_eq!(store.visible(), [] as [Todo; 0]);
        assert_eq!(store.active_count(), 0);
        assert_eq!(store.footer_text().get(), "All done!");
    }

    #[test]
    fn blank_drafts_are_ignored() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.set_draft("   ".to_string());
        store.commit_draft();
        assert_eq!(store.visible(), [] as [Todo; 0]);
        // Draft was preserved for editing (not cleared on blank commit).
        assert_eq!(store.draft.get(), "   ");
    }

    #[test]
    fn commit_clears_draft_and_increments_ids() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.add("buy milk");
        store.add("write code");
        let items = store.visible();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, 1);
        assert_eq!(items[1].id, 2);
        assert_eq!(items[1].title, "write code");
        assert_eq!(store.draft.get(), "");
    }

    #[test]
    fn toggle_and_filters() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.add("a");
        store.add("b");
        assert!(store.toggle(1));
        assert!(!store.toggle(99), "unknown id");

        store.set_filter(Filter::Active);
        assert_eq!(store.visible().len(), 1, "only 'b' is active");
        store.set_filter(Filter::Completed);
        assert_eq!(store.visible().len(), 1, "only 'a' is completed");
        assert_eq!(store.active_count(), 1);

        assert!(
            !store.set_filter(Filter::Completed),
            "no-op filter returns false"
        );
    }

    #[test]
    fn remove_and_clear_completed() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.add("x");
        store.add("y");
        store.toggle(1);
        store.clear_completed();
        assert_eq!(store.visible().len(), 1);
        assert!(store.remove(2));
        assert!(!store.remove(2), "double remove returns false");
        assert_eq!(store.visible(), [] as [Todo; 0]);
    }

    #[test]
    fn toggle_all_is_idempotent() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.add("a");
        // Already nothing to do: no change, no extra update churn.
        let before = cx.updates().count();
        store.set_all_completed(false);
        assert_eq!(
            cx.updates().count(),
            before,
            "no-op toggle-all writes nothing"
        );

        store.set_all_completed(true);
        assert!(store.all_completed());
        store.set_all_completed(true);
        assert!(store.all_completed());
    }

    #[test]
    fn set_eq_keeps_draft_writes_quiet() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let store = TodoStore::new(&cx);
        store.set_draft("same".to_string());
        let before = cx.updates().count();
        store.set_draft("same".to_string());
        assert_eq!(
            cx.updates().count(),
            before,
            "identical draft write is skipped"
        );
    }

    #[test]
    fn child_token_indicator_is_cancelled_with_parent() {
        let parent = CancellationToken::new();
        let indicator = SaveIndicator::new(&parent);
        assert!(!indicator.token.is_cancelled());
        parent.cancel();
        assert!(indicator.token.is_cancelled());
    }
}
