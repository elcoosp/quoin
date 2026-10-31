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
        if let Some(waker) = self.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
}

#[derive(Debug)]
struct Inner {
    cancelled: std::sync::atomic::AtomicBool,
    waiters: Mutex<Vec<Weak<WaiterNode>>>,
}

impl Inner {
    fn new() -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(false),
            waiters: Mutex::new(Vec::new()),
        }
    }

    fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let waiters = std::mem::take(&mut *self.waiters.lock().unwrap());
        for waiter in waiters {
            if let Some(node) = waiter.upgrade() {
                node.wake();
            }
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Registers a new waiter node, pruning entries of futures that were
    /// dropped before cancellation.
    fn register(self: &Arc<Self>, node: &Arc<WaiterNode>) {
        let mut waiters = self.waiters.lock().unwrap();
        waiters.retain(|weak| weak.strong_count() > 0);
        waiters.push(Arc::downgrade(node));
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
    pub fn cancelled(&self) -> Cancelled<'_> {
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
