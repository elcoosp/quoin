//! Cooperative cancellation primitives for asynchronous tasks.
//!
//! This module provides [`CancellationToken`], a lightweight, cloneable token
//! that can be used to signal cancellation to spawned futures. It is designed
//! to integrate seamlessly with the async executors provided by UI frameworks.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

#[derive(Debug)]
struct Inner {
    cancelled: std::sync::atomic::AtomicBool,
    wakers: Mutex<Vec<Waker>>,
}

impl Inner {
    fn new() -> Self {
        Self {
            cancelled: std::sync::atomic::AtomicBool::new(false),
            wakers: Mutex::new(Vec::new()),
        }
    }

    fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let wakers = std::mem::take(&mut *self.wakers.lock().unwrap());
        for waker in wakers {
            waker.wake();
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn register(&self, waker: &Waker) {
        self.wakers.lock().unwrap().push(waker.clone());
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
/// use quoin::CancellationToken;
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
    /// use quoin::CancellationToken;
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
    /// use quoin::CancellationToken;
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
    /// use quoin::CancellationToken;
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
    /// [`cancel`]: CancellationToken::cancel
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use quoin::CancellationToken;
    /// # async {
    /// let token = CancellationToken::new();
    /// // ... later
    /// token.cancelled().await;
    /// println!("Token was cancelled");
    /// # };
    /// ```
    #[must_use]
    pub fn cancelled(&self) -> Cancelled<'_> {
        Cancelled { token: self }
    }
}

/// Future that resolves when the associated [`CancellationToken`] is cancelled.
///
/// This future is created by the [`cancelled`] method on [`CancellationToken`].
///
/// [`cancelled`]: CancellationToken::cancelled
pub struct Cancelled<'a> {
    token: &'a CancellationToken,
}

impl Future for Cancelled<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            self.token.inner.register(cx.waker());
            Poll::Pending
        }
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
}
