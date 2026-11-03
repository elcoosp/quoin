//! Blanket extension traits that layer conveniences onto every adapter.
//!
//! These traits are implemented once, generically, over the core traits — so
//! **every** framework adapter gets them for free, with zero per-adapter
//! code:
//!
//! - [`SignalExt`] — cheap conditional writes ([`set_eq`](SignalExt::set_eq)),
//!   `replace`, `take`, and derived sugar ([`map`](SignalExt::map)).
//! - [`ExecutorExt`] — [`spawn_with_token`](ExecutorExt::spawn_with_token) for
//!   the fetch-and-cancel pattern.
//! - [`ReactiveContextExt`] — [`create_resource`](ReactiveContextExt::create_resource),
//!   [`batch`](ReactiveContextExt::batch), [`update_global`](ReactiveContextExt::update_global),
//!   and [`use_global_or_init`](ReactiveContextExt::use_global_or_init).

use crate::batch;
use crate::cancellation::CancellationToken;
use crate::executor::Executor;
use crate::reactive::ReactiveContext;
use crate::resource::{Resource, ResourceState};
use crate::signal::Signal;
use std::future::Future;

/// Extra operations available on every [`Signal`].
///
/// All methods have default implementations with no per-adapter overhead, so
/// `use quoin_core::SignalExt;` is all an application needs.
pub trait SignalExt<T: Clone + 'static>: Signal<T> {
    /// Sets the value only when it differs from the current one.
    ///
    /// On pull-based adapters (GPUI, Xilem) a redundant `set` still triggers
    /// a re-render request; skipping equal writes removes that wasted work.
    /// Returns `true` when the value was written.
    ///
    /// Delegates to [`Signal::set_if_eq`], which single-lock implementations
    /// override for a cheaper compare-and-write.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::prelude::*;
    ///
    /// let s = quoin_core::test_support::static_signal("a".to_string());
    /// assert!(s.set_eq("a".to_string()) == false); // unchanged: skipped
    /// assert!(s.set_eq("b".to_string()));          // changed: written
    /// assert_eq!(s.get(), "b");
    /// ```
    fn set_eq(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        self.set_if_eq(value)
    }

    /// Replaces the value and returns the previous one.
    fn replace(&self, value: T) -> T {
        let previous = self.get();
        self.set(value);
        previous
    }

    /// Takes the value out, leaving `T::default()` in its place.
    fn take(&self) -> T
    where
        T: Default,
    {
        self.replace(T::default())
    }

    /// Convenience for creating a [`Derived`](crate::Derived) signal from
    /// this one.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::prelude::*;
    ///
    /// let count = quoin_core::test_support::static_signal(2u32);
    /// let label = count.map(|n| format!("count = {n}"));
    /// assert_eq!(label.get(), "count = 2");
    /// ```
    fn map<U, F>(&self, f: F) -> crate::Derived<Self, T, U, F>
    where
        Self: Sized,
        T: Clone + 'static,
        U: Clone + 'static,
        F: Fn(&T) -> U + Clone + 'static,
    {
        crate::derived(self, f)
    }
}

impl<S, T> SignalExt<T> for S
where
    S: Signal<T>,
    T: Clone + 'static,
{
}

/// Extra operations available on every [`Executor`].
pub trait ExecutorExt: Executor {
    /// Spawns a future together with a fresh [`CancellationToken`].
    ///
    /// The closure receives the token so the task can poll
    /// `token.is_cancelled()` or `select` on `token.cancelled()`. Aborting
    /// the returned handle stops the task at its next suspension point; the
    /// token offers *cooperative* checkpoints inside the task body.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use quoin_core::prelude::*;
    ///
    /// let executor = quoin_core::test_support::StaticExecutor;
    /// let (handle, token) = executor.spawn_with_token(|token| async move {
    ///     while !token.is_cancelled() {
    ///         // ... do one unit of work ...
    /// #       break;
    ///     }
    /// });
    /// token.cancel();
    /// ```
    fn spawn_with_token<F, Fut>(
        &self,
        make_future: F,
    ) -> (Self::JoinHandle<Fut::Output>, CancellationToken)
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let token = CancellationToken::new();
        let future = make_future(token.clone());
        (self.spawn(future), token)
    }
}

impl<E: Executor> ExecutorExt for E {}

/// Extra operations available on every [`ReactiveContext`].
pub trait ReactiveContextExt: ReactiveContext {
    /// Creates a resource that immediately starts `fetch` on this context's
    /// executor.
    ///
    /// See the [`resource`](crate::resource) module for the full lifecycle
    /// documentation.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::prelude::*;
    ///
    /// let cx = quoin_core::test_support::StaticContext;
    /// let user = cx.create_resource(|_token| async {
    ///     Ok::<&str, String>("alice")
    /// });
    /// assert_eq!(user.read(), Some("alice"));
    /// ```
    fn create_resource<F, Fut, T, E>(
        &self,
        fetch: F,
    ) -> Resource<Self::Signal<ResourceState<T, E>>, T, E>
    where
        Self: Sized,
        T: Clone + Send + 'static,
        // `E` is stored inside a shared signal (`ResourceState::Failed`),
        // so it must be `Sync` too — same contract as the global store.
        E: Clone + Send + Sync + 'static,
        F: Fn(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        Self::Executor: Executor<JoinHandle<()>: crate::JoinHandle<()> + 'static>,
        Self::Signal<ResourceState<T, E>>: Send,
    {
        let resource = Resource::with_state(self.create_signal(ResourceState::Idle));
        resource.reload_with(self, fetch);
        resource
    }

    /// Mutates several signals while suppressing redundant update requests,
    /// then requests a single UI update.
    ///
    /// On fine-grained frameworks this is a no-op wrapper (their reactive
    /// graphs batch internally); on GPUI and Xilem it collapses N notifies
    /// into 1.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::prelude::*;
    ///
    /// let cx = quoin_core::test_support::StaticContext;
    /// let a = cx.create_signal(0u32);
    /// let b = cx.create_signal(String::new());
    /// cx.batch(|| {
    ///     a.set(10);
    ///     b.set("done".into());
    /// });
    /// assert_eq!(a.get(), 10);
    /// assert_eq!(b.get(), "done");
    /// ```
    fn batch<R>(&self, f: impl FnOnce() -> R) -> R {
        batch::with(self, f)
    }

    /// Mutates a provided global in place.
    ///
    /// Returns `false` (and does nothing) when no global of type `T` has
    /// been provided.
    fn update_global<T, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R>
    where
        T: Clone + Send + Sync + 'static,
    {
        let signal = self.use_global::<T>()?;
        let mut result = None;
        // `update` needs `&mut T`; wrap the result assignment inside.
        signal.update(|value| {
            result = Some(f(value));
        });
        result
    }

    /// Returns the global of type `T`, providing `init` when missing.
    ///
    /// Guarantees a usable signal on the first call, which removes the
    /// `if let Some(...)` dance for lazily-initialized app state.
    fn use_global_or_init<T>(&self, init: impl FnOnce() -> T) -> Self::Signal<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let existing = self.use_global::<T>();
        if existing.is_none() {
            self.provide_global(init());
        }
        // After the possible `provide_global`, the global must be present.
        existing
            .or_else(|| self.use_global::<T>())
            .expect("global present after provide_global")
    }
}

impl<C: ReactiveContext> ReactiveContextExt for C {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::static_signal;

    #[test]
    fn set_eq_reports_whether_it_wrote() {
        let signal = static_signal(String::from("a"));
        assert!(!signal.set_eq("a".to_string()), "equal write skipped");
        assert_eq!(signal.get(), "a");
        assert!(signal.set_eq("b".to_string()), "changed write lands");
        assert_eq!(signal.get(), "b");
    }

    #[test]
    fn set_eq_matches_set_if_eq_default_semantics() {
        // The generic default (read-then-write) must agree with the
        // single-lock override in `StaticSignal`.
        let overridden = static_signal(0u32);
        let default_impl = static_signal(0u32);
        assert_eq!(overridden.set_eq(0), default_impl.set_if_eq(0));
        assert_eq!(overridden.get(), default_impl.get());
        assert_eq!(overridden.set_eq(1), default_impl.set_if_eq(1));
        assert_eq!(overridden.get(), default_impl.get());
        assert_eq!(overridden.set_eq(1), default_impl.set_if_eq(1));
        assert_eq!(overridden.get(), default_impl.get());
    }

    #[test]
    fn replace_returns_previous_value() {
        let signal = static_signal(10u32);
        assert_eq!(signal.replace(20), 10);
        assert_eq!(signal.get(), 20);
    }

    #[test]
    fn take_leaves_default_behind() {
        let signal = static_signal(String::from("payload"));
        assert_eq!(signal.take(), "payload");
        assert_eq!(signal.get(), "");
    }

    #[test]
    fn map_derives_a_view_of_the_signal() {
        let count = static_signal(2u32);
        let label = count.map(|n| format!("count = {n}"));
        assert_eq!(label.get(), "count = 2");
        count.set(3);
        assert_eq!(label.get(), "count = 3", "derived re-reads its source");
    }
}
