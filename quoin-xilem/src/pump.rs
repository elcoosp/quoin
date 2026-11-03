//! The Xilem rebuild pump.
//!
//! Xilem has no reactive graph: the app logic re-runs only after an action.
//! A quoin signal written from a worker thread (or a timer) must therefore
//! request a rebuild explicitly. The pump does that with one atomic flag and
//! one in-flight message, so a burst of `N` writes produces exactly one
//! rebuild:
//!
//! 1. [`pump_view`] is included in the app's view tree. Its `worker` view
//!    stores the [`MessageProxy`] in this module on startup.
//! 2. [`notify`] — installed as the [`XilemContext`](crate::XilemContext)
//!    update notifier — flips the dirty flag and sends one pump message only
//!    when the flag flipped from false to true.
//! 3. The worker's response handler clears the flag *before* the rebuild the
//!    driver runs afterwards; writes landing during the rebuild set the flag
//!    again and coalesce into the next rebuild.
//!
//! The flag is a relaxed `AtomicBool`: no locks, no allocations per write.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use xilem::WidgetView;
use xilem::core::{MessageProxy, MessageResult, fork};
use xilem::view::{canvas, worker};

/// The pump message: content-free, `AnyDebug + Send`.
#[derive(Debug)]
pub struct PumpMsg;

static DIRTY: AtomicBool = AtomicBool::new(false);
static PROXY: Mutex<Option<MessageProxy<PumpMsg>>> = Mutex::new(None);

/// Stores the proxy the pump view receives on startup.
pub fn install(proxy: MessageProxy<PumpMsg>) {
    if let Ok(mut slot) = PROXY.lock() {
        *slot = Some(proxy);
    }
}

/// Requests a rebuild: flips the dirty flag and sends one message when the
/// flag transitioned false → true. Cheap to call from any thread.
pub fn notify() {
    if !DIRTY.swap(true, Ordering::AcqRel)
        && let Ok(guard) = PROXY.lock()
        && let Some(proxy) = guard.as_ref()
        && proxy.message(PumpMsg).is_err()
    {
        // The proxy is dead (app tearing down); the next pump view
        // install will re-arm. Nothing else to do here.
    }
}

/// Clears the dirty flag; called by the pump worker before the driver's
/// rebuild, so writes landing during the rebuild coalesce.
fn clear_dirty() {
    DIRTY.store(false, Ordering::Release);
}

/// Whether a rebuild was requested since the last clear (test hook).
#[must_use]
pub fn is_dirty() -> bool {
    DIRTY.load(Ordering::Acquire)
}

/// Installs [`notify`] as the context's update notifier.
pub fn install_notifier(ctx: &crate::XilemContext) {
    ctx.set_update_notifier(notify);
}

/// The pump view: include it once in the app's view tree (a hidden
/// zero-cost `worker` view that wires the proxy and drives rebuilds).
///
/// `worker` itself is a side-effect-only view (`Element = NoElement`), so it
/// is [`fork`](xilem::core::fork)ed with a zero-size [`canvas`] widget — the
/// canvas paints nothing and occupies no space, while the worker runs
/// alongside it inside the widget tree.
#[must_use]
pub fn pump_view<State>() -> impl WidgetView<State>
where
    State: 'static,
{
    fork(
        // The visible half: a widget that draws nothing and takes no space.
        canvas(|_state, _ctx, _scene, _size| {}),
        // The pump: installs the proxy and coalesces notifications.
        worker(
            // init_future: owns nothing (const-fn assert); stores the proxy.
            |proxy, _rx| {
                install(proxy);
                std::future::ready(())
            },
            // store_sender: nothing to store on the state.
            |_state: &mut State, _sender: tokio::sync::mpsc::UnboundedSender<()>| {},
            // on_response: clear the flag, then request the rebuild through a
            // benign action. `()` is the default Action.
            |_state: &mut State, _msg: PumpMsg| {
                clear_dirty();
            },
        ),
    )
}

/// A variant for apps whose `Action` is not `()`: the pump returns the
/// action the caller's `on_response` picks. Kept for the generic
/// `run_app!` push.
pub fn pump_view_with<State, Action>(
    make_action: impl Fn() -> Action + Send + Sync + 'static,
) -> impl WidgetView<State, Action>
where
    State: 'static,
    Action: 'static,
{
    fork(
        canvas(|_state, _ctx, _scene, _size| {}),
        worker(
            |proxy, _rx: tokio::sync::mpsc::UnboundedReceiver<()>| {
                install(proxy);
                std::future::ready(())
            },
            |_state: &mut State, _sender| {},
            move |_state: &mut State, _msg: PumpMsg| {
                clear_dirty();
                make_action()
            },
        ),
    )
}

/// Ensures the pump responds with `MessageResult::Action` semantics — the
/// worker view wraps the returned action automatically; this constant
/// documents the contract for the driver.
pub const PUMP_ACTION: MessageResult<()> = MessageResult::Action(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_flag_flips_and_clears() {
        assert!(!is_dirty());
        // notify without a proxy only flips the flag (no proxy yet).
        notify();
        assert!(is_dirty());
        clear_dirty();
        assert!(!is_dirty());
    }
}
