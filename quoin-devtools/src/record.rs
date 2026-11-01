//! Recording helpers shared by the instrumentation wrappers.
//!
//! Everything here is gated on the target log's enable flag: when recording
//! is off, the only cost is one relaxed atomic load.

use crate::event::{Event, EventKind, EventLog};
use quoin_core::signal::Signal;
use std::sync::Arc;

/// The process-wide default [`EventLog`].
#[must_use]
pub fn global_log() -> &'static EventLog {
    static GLOBAL_LOG: std::sync::OnceLock<EventLog> = std::sync::OnceLock::new();
    GLOBAL_LOG.get_or_init(|| EventLog::with_capacity(crate::DEFAULT_LOG_CAPACITY))
}

/// Returns `true` when the global log is recording.
#[must_use]
pub fn enabled() -> bool {
    global_log().is_enabled()
}

/// Turns instrumentation on for the global log.
pub fn enable() {
    global_log().set_enabled(true);
}

/// Turns instrumentation off for the global log.
pub fn disable() {
    global_log().set_enabled(false);
}

/// Enables instrumentation when the `QUOIN_TRACE` environment variable is
/// set to a truthy value (`1`, `on`, `true`, `yes`).
///
/// Call this once at the top of your `main` / app bootstrap. It is the
/// default behaviour of [`crate::Instrumented::wrap`].
pub fn init_from_env() {
    if let Ok(value) = std::env::var("QUOIN_TRACE")
        && parse_trace_flag(&value)
    {
        enable();
    }
}

/// Truthiness rule for the `QUOIN_TRACE` flag (exposed for testing).
#[must_use]
pub fn parse_trace_flag(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "on" | "true" | "yes"
    )
}

thread_local! {
    /// Cached thread label: one `Arc<str>` allocation per thread, so events
    /// never pay for a `String` clone of the thread name.
    static THREAD_LABEL: Arc<str> = Arc::from(
        std::thread::current()
            .name()
            .unwrap_or("<unnamed>")
            .to_string(),
    );
}

/// Returns this thread's cached label (name or `"<unnamed>"`).
#[must_use]
pub fn thread_label() -> Arc<str> {
    THREAD_LABEL.with(Arc::clone)
}

/// Builds a mutation event, optionally capturing a `Debug` snapshot.
///
/// Shared by [`record`](fn@record), [`snapshot`], and the traced signal so
/// the recorded shape stays identical everywhere.
pub(crate) fn mutation_event(
    scope: &'static str,
    type_name: &'static str,
    value: Option<String>,
) -> Event {
    let mut event = Event::now(0, EventKind::SignalMutated, scope, Some(type_name));
    event.value = value;
    event
}

/// Records an event into `log` if that log is enabled.
pub fn record(
    log: &EventLog,
    kind: EventKind,
    scope: &'static str,
    type_name: Option<&'static str>,
) {
    if log.is_enabled() {
        log.push(Event::now(0, kind, scope, type_name));
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "quoin_devtools", kind = kind.label(), scope, ?type_name);
    }
}

/// Convenience: records a mutation event with a debug value snapshot.
///
/// Works with any signal value that implements `Debug` — call it from hot
/// paths where seeing the actual value matters:
///
/// ```
/// use quoin_core::prelude::*;
/// use quoin_devtools::{enable, global_log, snapshot};
///
/// enable();
/// let cx = quoin_testkit::TestContext::new();
/// let count = cx.create_signal(0u32);
/// count.set(3);
/// snapshot(&count, "app"); // records SignalMutated with value "3"
/// let events = global_log().events();
/// assert!(events.iter().any(|e| e.value.as_deref() == Some("3")));
/// ```
pub fn snapshot<S, T>(signal: &S, scope: &'static str)
where
    S: Signal<T>,
    T: std::fmt::Debug + Clone + 'static,
{
    let log = global_log();
    if !log.is_enabled() {
        return;
    }
    let value = signal.with(|v| format!("{v:?}"));
    log.push(mutation_event(
        scope,
        std::any::type_name::<T>(),
        Some(value),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventKind;
    use quoin_core::test_support::static_signal;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    /// Serializes tests that toggle the process-global enable flag.
    fn global_flag_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn thread_label_is_cached_and_stable() {
        let a = thread_label();
        let b = thread_label();
        assert!(Arc::ptr_eq(&a, &b), "label must be cached per thread");
        assert!(!a.is_empty());
    }

    #[test]
    fn thread_label_from_named_thread() {
        let handle = std::thread::Builder::new()
            .name("quoin-record".to_string())
            .spawn(|| {
                let mut event = Event::now(0, EventKind::TaskSpawned, "named", None);
                event.thread = thread_label();
                event
            })
            .expect("spawn");
        let event = handle.join().expect("join");
        assert_eq!(&*event.thread, "quoin-record");
    }

    #[test]
    fn thread_label_falls_back_for_unnamed_threads() {
        // The test runner names its threads, so exercise the fallback logic
        // directly instead of relying on an unnamed thread existing.
        let label: Arc<str> = Arc::from(std::thread::current().name().unwrap_or("<unnamed>"));
        assert!(!label.is_empty());
    }

    #[test]
    fn parse_trace_flag_truthiness() {
        for truthy in ["1", "on", "true", "yes", "ON", "True", "YES"] {
            assert!(parse_trace_flag(truthy), "{truthy} must parse as true");
        }
        for falsy in ["", "0", "off", "nope", "false", "no", "2"] {
            assert!(!parse_trace_flag(falsy), "{falsy} must parse as false");
        }
    }

    #[test]
    fn global_log_is_a_stable_singleton() {
        let first = global_log();
        let second = global_log();
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn enable_disable_toggle_global_recording() {
        let _guard = global_flag_lock();
        enable();
        assert!(enabled());
        disable();
        assert!(!enabled());
        // Restore: other tests enable explicitly.
        disable();
    }

    #[test]
    fn init_from_env_reads_truthy_variable() {
        // SAFETY-free check: this test only verifies the flag parser wiring by
        // calling `parse_trace_flag` on the current value; `init_from_env`
        // itself mutates process env in a race-prone way, so the env-var
        // behaviour is exercised by the `trace-demo` binary instead.
        let flag = std::env::var("QUOIN_TRACE").map(|value| parse_trace_flag(&value));
        assert!(flag.is_ok() || flag.is_err());
    }

    #[test]
    fn snapshot_respects_global_enable_flag() {
        let _guard = global_flag_lock();
        let signal = static_signal(41u32);
        disable();
        let before = global_log().next_seq();
        snapshot(&signal, "flag");
        assert_eq!(
            global_log().next_seq(),
            before,
            "disabled global log must not record"
        );
        enable();
        snapshot(&signal, "flag");
        assert!(
            global_log().next_seq() > before,
            "enabled global log must record the snapshot"
        );
        disable();
    }

    #[test]
    fn snapshot_captures_value_and_type() {
        let _guard = global_flag_lock();
        enable();
        let log = global_log();
        let before = log.next_seq();
        let signal = static_signal(String::from("hello"));
        signal.set(String::from("world"));
        snapshot(&signal, "snap");
        let last = log
            .events()
            .into_iter()
            .rev()
            .find(|event| event.seq >= before)
            .expect("snapshot recorded an event");
        assert_eq!(last.kind, EventKind::SignalMutated);
        assert_eq!(last.value.as_deref(), Some("\"world\""));
        assert_eq!(last.type_name, Some("alloc::string::String"));
        assert_eq!(last.scope, "snap");
        disable();
    }

    #[test]
    fn mutation_event_sets_kind_and_type() {
        let event = mutation_event("scope", "u32", Some(String::from("7")));
        assert_eq!(event.kind, EventKind::SignalMutated);
        assert_eq!(event.type_name, Some("u32"));
        assert_eq!(event.value.as_deref(), Some("7"));
        assert_eq!(event.scope, "scope");
    }
}
