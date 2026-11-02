//! Portable timers for the [`Executor`] contract.
//!
//! quoin adapters previously had no unified way to wait: `GpuiExecutor`
//! exposed an inherent `sleep`, the testkit shipped its own, and libraries
//! had to reach for runtime-specific crates. This module provides the
//! adapter-neutral fallback used by [`Executor::sleep`]
//! (see [`crate::executor`]):
//!
//! - **native targets** — [`futures_timer::Delay`], a dependency-free timer
//!   that runs on a background timer thread and therefore needs no ambient
//!   async runtime;
//! - **`wasm32`** — the browser timer through `gloo-timers`, driven on the
//!   main thread; the returned future stays `Send` by awaiting a `Send`
//!   oneshot channel instead of the `!Send` timer future itself.
//!
//! Executors may override [`Executor::sleep`] with their native timer (for
//! example the testkit `TokioExecutor` delegates to `tokio::time::sleep` so
//! paused-time tests advance virtual clocks), but the default makes every
//! adapter timer-capable out of the box.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// The boxed future returned by [`Executor::sleep`](crate::Executor::sleep)
/// and [`sleep`].
pub type SleepFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Resolves after `duration` of wall-clock time, without requiring any
/// ambient async runtime on native targets.
///
/// # Panics
/// Never panics. `wasm32` rounds durations above ~49 days down to the
/// browser's maximum `setTimeout` delay.
#[must_use = "sleep returns a future that must be awaited"]
pub fn sleep(duration: Duration) -> SleepFuture {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_sleep(duration)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::pin(futures_timer::Delay::new(duration))
    }
}

/// wasm32 implementation: schedule a browser timeout on the main thread and
/// await its completion over a `Send` channel.
///
/// `gloo_timers::future::TimeoutFuture` is `!Send` (it owns a JS closure), so
/// it is driven by `wasm_bindgen_futures::spawn_local` — which accepts
/// `!Send` futures and runs them on the single wasm thread — while the
/// caller only holds the `Send` oneshot receiver.
#[cfg(target_arch = "wasm32")]
fn wasm_sleep(duration: Duration) -> SleepFuture {
    let (tx, rx) = futures_channel::oneshot::channel::<()>();
    // Browsers cap `setTimeout` delays at 2^31-1 ms; clamp so the rounding
    // stays lossless.
    let millis = duration.as_millis().min(i32::MAX as u128) as i32;
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(millis).await;
        let _ = tx.send(());
    });
    Box::pin(async move {
        let _ = rx.await;
    })
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;

    #[test]
    fn sleep_resolves_after_duration() {
        let start = Instant::now();
        crate::pool::block_on(sleep(Duration::from_millis(20)));
        assert!(
            start.elapsed() >= Duration::from_millis(18),
            "sleep must wait at least the requested duration"
        );
    }

    #[test]
    fn sleep_future_is_send_and_composable() {
        // Compile-time: SleepFuture must remain Send so spawned tasks can
        // await timers (the core contract for `Executor::sleep`).
        fn assert_send<T: Send>(_: &T) {}
        let fut = sleep(Duration::from_millis(1));
        assert_send(&fut);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = Arc::clone(&done);
        let fut = sleep(Duration::from_millis(1));
        crate::pool::spawn_pooled(async move {
            fut.await;
            done_clone.store(true, Ordering::SeqCst);
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !done.load(Ordering::SeqCst) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(
            done.load(Ordering::SeqCst),
            "pooled task must observe the sleep"
        );
    }
}
