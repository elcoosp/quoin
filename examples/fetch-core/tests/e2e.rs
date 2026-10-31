//! End-to-end async lifecycle tests for the search page.
//!
//! `ImmediateExecutor` covers the deterministic paths (instant resolution,
//! validation, error mapping). The `ThreadedExecutor` + latency backend
//! covers real concurrency: reload cancellation, stale-generation
//! suppression, and load/error transitions.

use fetch_core::{FetchError, SearchPage, memory_backend, with_latency};
use quoin_core::prelude::*;
use quoin_testkit::{ImmediateExecutor, TestContext, ThreadedExecutor};

#[test]
fn e2e_submit_reloads_and_updates_items() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let backend = memory_backend(vec![
        ("rust".into(), vec!["book".into(), "crate".into()]),
        ("quoin".into(), vec!["one core".into()]),
    ]);
    let page = SearchPage::new(&cx, "rust", backend);

    assert_eq!(page.items().len(), 2);

    // The user edits the query and presses Enter.
    page.set_query("quoin".into());
    page.submit(&cx);

    assert_eq!(page.items(), vec!["one core"]);
    assert!(!page.is_loading());
    assert!(page.error().is_none());
}

#[test]
fn e2e_validation_and_error_mapping() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let page = SearchPage::new(&cx, "", memory_backend(vec![]));
    assert_eq!(
        page.error(),
        Some(FetchError::InvalidQuery("".into())),
        "empty initial query must surface the validation error"
    );
    assert!(page.items().is_empty());
}

#[test]
fn e2e_generation_suppresses_stale_results() {
    // Two submits in a row: the first generation must never overwrite the
    // second, even though both futures are created immediately.
    let cx = TestContext::with_executor(ImmediateExecutor);
    let backend = memory_backend(vec![
        ("first".into(), vec!["old result".into()]),
        ("second".into(), vec!["new result".into()]),
    ]);
    let page = SearchPage::new(&cx, "first", backend);

    page.set_query("second".into());
    page.submit(&cx);
    assert_eq!(page.items(), vec!["new result"]);
    assert!(page.results.generation() >= 2);
}

#[test]
fn e2e_cancel_moves_resource_to_idle() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let page = SearchPage::new(
        &cx,
        "rust",
        memory_backend(vec![("rust".into(), vec!["x".into()])]),
    );
    assert!(page.results.read().is_some());
    page.results.cancel();
    assert!(matches!(page.results.snapshot(), ResourceState::Idle));
    assert!(page.items().is_empty());
}

#[test]
fn e2e_latency_backend_reports_loading_then_ready() {
    let cx = TestContext::with_executor(ThreadedExecutor);
    let backend = with_latency(
        memory_backend(vec![("slow".into(), vec!["late".into()])]),
        30,
    );
    let page = SearchPage::new(&cx, "slow", backend);

    // Submit a fresh request and observe the Loading phase from the reload.
    page.set_query("slow".into());
    let query_signal = page.query.clone();
    let _ = query_signal;
    page.submit(&cx);
    assert!(
        page.is_loading(),
        "freshly submitted request must be loading"
    );

    // Wait for completion (bounded so a regression fails fast, not hangs).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while page.is_loading() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        !page.is_loading(),
        "request must complete within the deadline"
    );
    assert_eq!(page.items(), vec!["late"]);
}

#[test]
fn e2e_reload_cancels_in_flight_request() {
    // A slow backend plus a rapid double submit: only the newest generation
    // may publish; the stale one is cancelled mid-flight.
    let cx = TestContext::with_executor(ThreadedExecutor);
    let backend = with_latency(
        memory_backend(vec![
            ("one".into(), vec!["1".into()]),
            ("two".into(), vec!["2".into()]),
        ]),
        200,
    );
    let page = SearchPage::new(&cx, "one", backend);

    page.set_query("two".into());
    page.submit(&cx);
    page.submit(&cx); // cancels the first request, starts a second

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while page.is_loading() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(!page.is_loading());
    assert_eq!(
        page.items(),
        vec!["2"],
        "only the newest generation publishes"
    );
}

#[test]
fn e2e_resource_is_cloneable_across_holders() {
    let cx = TestContext::with_executor(ImmediateExecutor);
    let page = SearchPage::new(
        &cx,
        "rust",
        memory_backend(vec![("rust".into(), vec!["q".into()])]),
    );
    let state_clone = page.results.state();
    assert_eq!(state_clone.get().as_ready().map(|r| r.items.len()), Some(1));
}
