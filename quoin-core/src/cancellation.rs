//! Cooperative cancellation primitives for asynchronous tasks.
//!
//! This module provides [`CancellationToken`], a lightweight, cloneable token
//! that can be used to signal cancellation to spawned futures. It is designed
//! to integrate seamlessly with the async executors provided by UI frameworks.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll, Waker};

/// A single waiter registered by a `Cancelled` future.
///
/// The node owns the most recent waker of the future that created it. The
/// future updates the waker in place on re-poll, so repeated polling never
/// accumulates new registry entries.
struct WaiterNode {
    waker: Mutex<Option<Waker>>,
}

impl WaiterNode {
    fn new(waker: &Waker) -> Self {
        Self {
            waker: Mutex::new(Some(waker.clone())),
        }
    }

    fn update_waker(&self, waker: &Waker) {
        let mut slot = self.waker.lock().unwrap();
        match &*slot {
            Some(existing) if existing.will_wake(waker) => {}
            _ => *slot = Some(waker.clone()),
        }
    }

    fn wake(self: Arc<Self>) {
        // Take the waker while holding the lock, then wake after the guard
        // is gone: `Waker::wake` may synchronously re-enter this type.
        let waker = self.waker.lock().unwrap().take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

#[derive(Debug)]
struct Inner {
    cancelled: std::sync::atomic::AtomicBool,
    waiters: Mutex<Vec<Weak<WaiterNode>>>,
    /// Child tokens created via [`CancellationToken::child_token`]. Cancelled
    /// transitively when this token is cancelled.
    children: Mutex<Vec<Weak<Self>>>,
}

impl Inner {
    const fn new() -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(false),
            waiters: Mutex::new(Vec::new()),
            children: Mutex::new(Vec::new()),
        }
    }

    fn cancel(self: &Arc<Self>) {
        if self
            .cancelled
            .swap(true, std::sync::atomic::Ordering::AcqRel)
        {
            // Already cancelled: avoid double-waking and cascading twice.
            return;
        }
        // Wake local waiters.
        let waiters = std::mem::take(&mut *self.waiters.lock().unwrap());
        for waiter in waiters {
            if let Some(node) = waiter.upgrade() {
                node.wake();
            }
        }
        // Cascade to children (each child.cancel propagates further down).
        let children = std::mem::take(&mut *self.children.lock().unwrap());
        for child in children {
            if let Some(child) = child.upgrade() {
                child.cancel();
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Registers a new waiter node, pruning entries of futures that were
    /// dropped before cancellation.
    fn register(self: &Arc<Self>, node: &Arc<WaiterNode>) {
        let mut waiters = self.waiters.lock().unwrap();
        waiters.retain(|weak| weak.strong_count() > 0);
        waiters.push(Arc::downgrade(node));
    }

    /// Registers a child token, pruning dropped children.
    fn register_child(self: &Arc<Self>, child: &Arc<Self>) {
        let mut children = self.children.lock().unwrap();
        children.retain(|weak| weak.strong_count() > 0);
        children.push(Arc::downgrade(child));
    }
}

/// A token that can be used for cooperative cancellation.
///
/// `CancellationToken` is a cheaply cloneable handle that can be shared across
/// tasks. When cancelled, all associated [`cancelled`] futures are notified
/// and resolve immediately.
///
/// # Example
///
/// ```rust,ignore
/// use quoin_core::CancellationToken;
///
/// let token = CancellationToken::new();
/// let clone = token.clone();
///
/// // Spawn a task that checks for cancellation
/// let handle = tokio::spawn(async move {
///     loop {
///         tokio::select! {
///             _ = clone.cancelled() => {
///                 println!("Cancelled!");
///                 break;
///             }
///             _ = tokio::time::sleep(Duration::from_millis(100)) => {
///                 // Do work...
///             }
///         }
///     }
/// });
///
/// // Cancel the token after a while
/// token.cancel();
/// ```
///
/// [`cancelled`]: CancellationToken::cancelled
#[derive(Clone, Debug)]
pub struct CancellationToken {
    inner: Arc<Inner>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    /// Creates a new, uncancelled token.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    /// let token = CancellationToken::new();
    /// assert!(!token.is_cancelled());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner::new()),
        }
    }

    /// Cancels the token.
    ///
    /// This wakes all pending futures that are waiting on [`cancelled`].
    ///
    /// [`cancelled`]: CancellationToken::cancelled
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    /// let token = CancellationToken::new();
    /// token.cancel();
    /// assert!(token.is_cancelled());
    /// ```
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Returns `true` if the token has been cancelled.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    /// let token = CancellationToken::new();
    /// assert!(!token.is_cancelled());
    /// token.cancel();
    /// assert!(token.is_cancelled());
    /// ```
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }

    /// Creates a child token that is cancelled either on its own or when
    /// `self` is cancelled.
    ///
    /// Children form a tree: cancelling a parent cancels all descendants
    /// transitively, while a child (or any of its own children) can still be
    /// cancelled independently. Dropped children are pruned from the parent,
    /// so creating per-request child tokens does not leak.
    ///
    /// This is the standard pattern for scoped async work — e.g. one child
    /// per in-flight network request, all cancelled together when the view
    /// unmounts.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    ///
    /// let parent = CancellationToken::new();
    /// let child = parent.child_token();
    /// assert!(!child.is_cancelled());
    ///
    /// parent.cancel();
    /// assert!(child.is_cancelled(), "cancelling the parent cascades");
    /// ```
    #[must_use]
    pub fn child_token(&self) -> Self {
        let child_inner = Arc::new(Inner::new());
        self.inner.register_child(&child_inner);
        Self { inner: child_inner }
    }

    /// Returns a future that resolves when the token is cancelled,
    /// **consuming** the token so the future is `'static`.
    ///
    /// Use this when the waiting task needs to own its cancellation future
    /// (e.g. `tokio::select!` branches, or storing the future in a struct).
    /// The borrowed variant [`cancelled`](Self::cancelled) remains available
    /// for `&self` usage.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    ///
    /// # futures::executor::block_on(async {
    /// let token = CancellationToken::new();
    /// token.cancel();
    /// token.into_cancelled().await; // resolves immediately
    /// # });
    /// ```
    #[must_use]
    pub const fn into_cancelled(self) -> CancelledFuture {
        CancelledFuture {
            token: self,
            waiter: None,
        }
    }

    /// Returns an RAII guard that cancels the token when dropped.
    ///
    /// Useful for scope-bound cleanup: whatever the guard protects is torn
    /// down at the end of the block, even on early return or panic.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// {
    ///     let _guard = token.guard();
    ///     assert!(!token.is_cancelled());
    /// }
    /// assert!(token.is_cancelled(), "guard cancelled the token on drop");
    /// ```
    #[must_use]
    pub fn guard(&self) -> CancelGuard {
        CancelGuard {
            token: self.clone(),
        }
    }

    /// Races `future` against cancellation.
    ///
    /// Resolves with `Some(output)` when the future completes first, or
    /// `None` when the token is cancelled first. The future is dropped
    /// (and thus stopped) if it loses the race.
    ///
    /// # Example
    ///
    /// ```
    /// use quoin_core::CancellationToken;
    /// # futures::executor::block_on(async {
    /// let token = CancellationToken::new();
    /// let value = token.run_until_cancelled(async { 21 * 2 }).await;
    /// assert_eq!(value, Some(42));
    ///
    /// token.cancel();
    /// let aborted: Option<std::convert::Infallible> =
    ///     token.run_until_cancelled(std::future::pending()).await;
    /// assert_eq!(aborted, None);
    /// # });
    /// ```
    #[must_use]
    pub fn run_until_cancelled<F: Future>(&self, future: F) -> RunUntilCancelled<'_, F> {
        RunUntilCancelled {
            token: self,
            future: Box::pin(future),
            cancelled: None,
        }
    }

    /// Returns a future that resolves when the token is cancelled.
    ///
    /// If the token is already cancelled, the future resolves immediately.
    /// Otherwise, it waits until [`cancel`] is called.
    ///
    /// Dropping the returned future removes its wait registration; the token
    /// keeps no reference to futures that are no longer polled.
    ///
    /// [`cancel`]: CancellationToken::cancel
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use quoin_core::CancellationToken;
    /// # async {
    /// let token = CancellationToken::new();
    /// // ... later
    /// token.cancelled().await;
    /// println!("Token was cancelled");
    /// # };
    /// ```
    #[must_use]
    pub const fn cancelled(&self) -> Cancelled<'_> {
        Cancelled {
            token: self,
            waiter: None,
        }
    }
}

/// Future that resolves when the associated [`CancellationToken`] is cancelled.
///
/// This future is created by the [`cancelled`] method on [`CancellationToken`].
///
/// [`cancelled`]: CancellationToken::cancelled
pub struct Cancelled<'a> {
    token: &'a CancellationToken,
    /// The waiter node for this future. `None` until the first poll, so that
    /// creating the future is allocation-free and immediately-ready tokens do
    /// not register at all.
    waiter: Option<Arc<WaiterNode>>,
}

impl Future for Cancelled<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            // Drop the registration eagerly so the token forgets this future
            // as soon as it is done.
            self.waiter = None;
            return Poll::Ready(());
        }
        if let Some(node) = &self.waiter {
            node.update_waker(cx.waker());
        } else {
            let node = Arc::new(WaiterNode::new(cx.waker()));
            self.token.inner.register(&node);
            self.waiter = Some(node);
        }
        Poll::Pending
    }
}

/// Owned variant of [`Cancelled`]: resolves when the token is cancelled and
/// holds the token itself, so it is `'static`.
///
/// Created by [`CancellationToken::into_cancelled`].
pub struct CancelledFuture {
    token: CancellationToken,
    waiter: Option<Arc<WaiterNode>>,
}

impl Future for CancelledFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            self.waiter = None;
            return Poll::Ready(());
        }
        if let Some(node) = &self.waiter {
            node.update_waker(cx.waker());
        } else {
            let node = Arc::new(WaiterNode::new(cx.waker()));
            self.token.inner.register(&node);
            self.waiter = Some(node);
        }
        Poll::Pending
    }
}

/// RAII guard created by [`CancellationToken::guard`]: cancels the token when
/// dropped.
pub struct CancelGuard {
    token: CancellationToken,
}

impl std::fmt::Debug for CancelGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancelGuard").finish_non_exhaustive()
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

/// Future returned by [`CancellationToken::run_until_cancelled`].
///
/// Resolves with `Some(output)` when the wrapped future finishes first, or
/// `None` when cancellation wins.
pub struct RunUntilCancelled<'a, F> {
    token: &'a CancellationToken,
    /// Boxed so the combinator works with `!Unpin` futures (compiler
    /// `async` blocks, timers, …) without requiring `unsafe` projection.
    future: Pin<Box<F>>,
    /// Lazily created on first poll so an already-cancelled token never
    /// allocates a waiter.
    cancelled: Option<Cancelled<'a>>,
}

impl<F: Future> Future for RunUntilCancelled<'_, F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        match this.future.as_mut().poll(cx) {
            Poll::Ready(output) => return Poll::Ready(Some(output)),
            Poll::Pending => {}
        }
        if this.token.is_cancelled() {
            return Poll::Ready(None);
        }
        if this.cancelled.is_none() {
            this.cancelled = Some(this.token.cancelled());
        }
        if Pin::new(this.cancelled.as_mut().expect("just set")).poll(cx) == Poll::Ready(()) {
            return Poll::Ready(None);
        }
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cancellation_token_basic() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone() {
        let token = CancellationToken::new();
        let clone = token.clone();

        clone.cancel();
        assert!(token.is_cancelled());
        assert!(clone.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_future_immediate() {
        let token = CancellationToken::new();
        token.cancel();
        token.cancelled().await; // Should not hang
    }

    #[tokio::test]
    async fn test_cancellation_future_waits() {
        let token = CancellationToken::new();
        let clone = token.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            clone.cancel();
        });

        token.cancelled().await;
        handle.await.unwrap();
        assert!(token.is_cancelled());
    }

    #[test]
    fn child_token_cancels_with_parent() {
        let parent = CancellationToken::new();
        let child = parent.child_token();
        let grandchild = child.child_token();

        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());
        assert!(!grandchild.is_cancelled());

        child.cancel();
        assert!(child.is_cancelled());
        assert!(grandchild.is_cancelled(), "cancel cascades downward");
        assert!(!parent.is_cancelled(), "cancel must not propagate upward");

        parent.cancel();
        assert!(parent.is_cancelled());
    }

    #[test]
    fn parent_cancel_cascades_to_all_children() {
        let parent = CancellationToken::new();
        let children: Vec<_> = (0..5).map(|_| parent.child_token()).collect();
        parent.cancel();
        for child in &children {
            assert!(child.is_cancelled());
        }
    }

    #[test]
    fn dropped_children_are_pruned() {
        let parent = CancellationToken::new();
        {
            let _child = parent.child_token();
        }
        // Creating one more child triggers the prune of the dropped one; the
        // registry must not grow unboundedly.
        for _ in 0..10 {
            let _ = parent.child_token();
        }
        assert_eq!(parent.inner.children.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn into_cancelled_resolves_when_cancelled() {
        let token = CancellationToken::new();
        let clone = token.clone();
        let fut = token.into_cancelled();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            clone.cancel();
        });
        fut.await;
    }

    #[tokio::test]
    async fn into_cancelled_immediate() {
        let token = CancellationToken::new();
        token.cancel();
        token.into_cancelled().await;
    }

    #[test]
    fn cancel_guard_cancels_on_drop() {
        let token = CancellationToken::new();
        {
            let _guard = token.guard();
            assert!(!token.is_cancelled());
        }
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn run_until_cancelled_completes_first() {
        let token = CancellationToken::new();
        let value = token.run_until_cancelled(async { 42u32 }).await;
        assert_eq!(value, Some(42));
    }

    #[tokio::test]
    async fn run_until_cancelled_loses_to_cancellation() {
        let token = CancellationToken::new();
        let clone = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            clone.cancel();
        });
        let result: Option<std::convert::Infallible> =
            token.run_until_cancelled(std::future::pending()).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn run_until_cancelled_with_already_cancelled_token() {
        let token = CancellationToken::new();
        token.cancel();
        let result: Option<std::convert::Infallible> =
            token.run_until_cancelled(std::future::pending()).await;
        assert_eq!(result, None);
    }

    #[test]
    fn double_cancel_is_idempotent() {
        let token = CancellationToken::new();
        let child = token.child_token();
        token.cancel();
        token.cancel();
        assert!(child.is_cancelled());
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_waker_registry_is_bounded() {
        // Repeatedly polling a `Cancelled` future must not grow the token's
        // waker registry: only one waiter node is registered per future.
        let token = CancellationToken::new();
        {
            let mut fut = Box::pin(token.cancelled());
            let waker = futures_task_noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);
            for _ in 0..1_000 {
                assert!(fut.as_mut().poll(&mut cx).is_pending());
            }
        }
        // The future was dropped; a fresh registration prunes it. After the
        // prune there must be exactly zero waiters left.
        {
            let mut fut = Box::pin(token.cancelled());
            let waker = futures_task_noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);
            assert!(fut.as_mut().poll(&mut cx).is_pending());
        }
        assert_eq!(token.inner.waiters.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_cancellation_wakes_multiple_waiters() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let token = CancellationToken::new();
        let woken = Arc::new(AtomicUsize::new(0));

        let mut futures: Vec<std::pin::Pin<Box<Cancelled<'_>>>> = Vec::new();
        let mut flags = Vec::new();
        for _ in 0..4 {
            let flag = Arc::new(AtomicUsize::new(0));
            let waker = futures_waker_on_flag(Arc::clone(&flag), Arc::clone(&woken));
            flags.push(flag);
            let mut cx = std::task::Context::from_waker(&waker);
            let mut fut = Box::pin(token.cancelled());
            assert!(fut.as_mut().poll(&mut cx).is_pending());
            futures.push(fut);
        }

        token.cancel();
        assert_eq!(woken.load(Ordering::SeqCst), 4);
        for flag in &flags {
            assert_eq!(flag.load(Ordering::SeqCst), 1);
        }
        for fut in &mut futures {
            let waker = futures_task_noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);
            assert!(fut.as_mut().poll(&mut cx).is_ready());
        }
    }

    fn futures_task_noop_waker() -> std::task::Waker {
        std::task::Waker::noop().clone()
    }

    fn futures_waker_on_flag(
        flag: Arc<std::sync::atomic::AtomicUsize>,
        total: Arc<std::sync::atomic::AtomicUsize>,
    ) -> std::task::Waker {
        use std::sync::atomic::Ordering;
        use std::task::Wake;
        struct FlagWaker {
            flag: Arc<std::sync::atomic::AtomicUsize>,
            total: Arc<std::sync::atomic::AtomicUsize>,
        }
        impl Wake for FlagWaker {
            fn wake(self: Arc<Self>) {
                self.flag.fetch_add(1, Ordering::SeqCst);
                self.total.fetch_add(1, Ordering::SeqCst);
            }
        }
        std::task::Waker::from(Arc::new(FlagWaker { flag, total }))
    }
}
