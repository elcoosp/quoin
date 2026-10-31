//! Async data-loading with [`quoin_core::Resource`] — the framework-agnostic
//! equivalent of Leptos' `Resource` / Dioxus' `use_resource`, e2e tested.
//!
//! The example models a search page: the user types a query, the app fetches
//! results, reloading cancels stale in-flight requests, and failures surface
//! as typed errors. The full lifecycle is asserted in `tests/e2e.rs` using
//! both the deterministic `ImmediateExecutor` and the real-threaded one.

use quoin_core::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// What the backend returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub query: String,
    pub items: Vec<String>,
}

/// Errors the fetch can produce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FetchError {
    /// Simulated network failure.
    Network(String),
    /// The query was rejected by validation.
    InvalidQuery(String),
}

/// The backend function a `SearchPage` runs: receives the query text and the
/// per-request cancellation token (real HTTP calls should check
/// `token.is_cancelled()` between awaits).
pub type FetchFn = Arc<dyn Fn(String, CancellationToken) -> FutFetch + Send + Sync + 'static>;

/// Boxed future type returned by [`FetchFn`].
pub type FutFetch = Pin<Box<dyn Future<Output = Result<SearchResult, FetchError>> + Send>>;

/// A search page: one text input plus one resource, the canonical async-UI
/// pattern. Generic over the adapter context, so the identical code runs on
/// GPUI, Leptos, Dioxus, Xilem, Floem, and the testkit.
pub struct SearchPage<C: ReactiveContext>
where
    C::Executor: Executor<JoinHandle<()>: JoinHandle<()> + 'static>,
    C::Signal<ResourceState<SearchResult, FetchError>>: Send,
{
    /// Current input value.
    pub query: C::Signal<String>,
    /// The in-flight/loaded resource.
    pub results:
        Resource<C::Signal<ResourceState<SearchResult, FetchError>>, SearchResult, FetchError>,
    fetch: FetchFn,
}

impl<C: ReactiveContext> SearchPage<C>
where
    C: ReactiveContext,
    C::Executor: Executor<JoinHandle<()>: JoinHandle<()> + 'static>,
    C::Signal<ResourceState<SearchResult, FetchError>>: Send,
{
    /// Builds the page and immediately issues the first fetch for
    /// `initial_query`.
    #[must_use]
    pub fn new(cx: &C, initial_query: &str, fetch: FetchFn) -> Self {
        let query = cx.create_signal(initial_query.to_string());
        let fetch_for_resource = fetch.clone();
        let initial = initial_query.to_string();
        let results = cx.create_resource(move |token| {
            let fetch = fetch_for_resource.clone();
            let query = initial.clone();
            Box::pin(async move { run_query(&fetch, query, token).await }) as FutFetch
        });
        Self {
            query,
            results,
            fetch,
        }
    }

    /// Updates the input text (skips no-op writes so text inputs stay quiet).
    pub fn set_query(&self, text: String) {
        self.query.set_eq(text);
    }

    /// Simulates pressing Enter: reloads the resource for the current query
    /// text. Any in-flight request is cancelled first — the resource
    /// guarantees only the newest generation can publish a result.
    pub fn submit(&self, cx: &C) {
        let text = self.query.get().trim().to_string();
        if text.is_empty() {
            return;
        }
        let fetch = self.fetch.clone();
        self.results.reload_with(cx, move |token| {
            let fetch = fetch.clone();
            let query = text.clone();
            Box::pin(async move { run_query(&fetch, query, token).await }) as FutFetch
        });
    }

    /// True while a request is in flight (bind to a spinner).
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.results.snapshot().is_loading()
    }

    /// The loaded items, empty while loading/failed.
    #[must_use]
    pub fn items(&self) -> Vec<String> {
        self.results.read().map(|r| r.items).unwrap_or_default()
    }

    /// The error of the latest failed attempt, if any.
    #[must_use]
    pub fn error(&self) -> Option<FetchError> {
        self.results.snapshot().as_failed().map(|e| (*e).clone())
    }
}

/// Shared request pipeline: validation, backend call, result tagging.
async fn run_query(
    fetch: &FetchFn,
    query: String,
    token: CancellationToken,
) -> Result<SearchResult, FetchError> {
    if query.is_empty() {
        return Err(FetchError::InvalidQuery(query));
    }
    let mut result = fetch(query.clone(), token).await?;
    result.query = query;
    Ok(result)
}

/// Builds a deterministic in-memory backend: queries listed in `table`
/// resolve with the given items, everything else fails with
/// [`FetchError::Network`].
#[must_use]
pub fn memory_backend(table: Vec<(String, Vec<String>)>) -> FetchFn {
    let table: Arc<std::collections::HashMap<String, Vec<String>>> =
        Arc::new(table.into_iter().collect());
    Arc::new(move |query, _token| {
        let table = table.clone();
        Box::pin(async move {
            match table.get(&query) {
                Some(items) => Ok(SearchResult {
                    query,
                    items: items.clone(),
                }),
                None => Err(FetchError::Network(format!("no results for '{query}'"))),
            }
        })
    })
}

/// Wraps a backend with simulated latency, racing the delay against
/// cancellation so aborted requests stop early.
#[must_use]
pub fn with_latency(inner: FetchFn, latency_ms: u64) -> FetchFn {
    Arc::new(move |query, token| {
        let inner = inner.clone();
        Box::pin(async move {
            let delay = futures_timer::Delay::new(std::time::Duration::from_millis(latency_ms));
            match token.run_until_cancelled(delay).await {
                Some(()) => inner(query, token).await,
                None => Err(FetchError::Network("cancelled".into())),
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quoin_testkit::{ImmediateExecutor, TestContext};

    #[test]
    fn memory_backend_resolves_known_queries() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let page = SearchPage::new(
            &cx,
            "rust",
            memory_backend(vec![("rust".to_string(), vec!["quoin".to_string()])]),
        );
        assert_eq!(page.items(), vec!["quoin"]);
        assert_eq!(page.results.read().map(|r| r.query), Some("rust".into()));
    }

    #[test]
    fn unknown_queries_fail_with_network_error() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let page = SearchPage::new(&cx, "nope", memory_backend(vec![]));
        assert!(page.items().is_empty());
        assert_eq!(
            page.error(),
            Some(FetchError::Network("no results for 'nope'".into()))
        );
    }

    #[test]
    fn set_query_skips_noop_writes() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let page = SearchPage::new(&cx, "a", memory_backend(vec![]));
        cx.updates().reset();
        page.set_query("a".into());
        assert_eq!(cx.updates().count(), 0);
        page.set_query("b".into());
        assert_eq!(cx.updates().count(), 1);
    }

    #[test]
    fn submit_with_empty_query_is_ignored() {
        let cx = TestContext::with_executor(ImmediateExecutor);
        let page = SearchPage::new(&cx, "", memory_backend(vec![]));
        // The initial fetch already rejected the empty query; submitting
        // again must not even start a request (generation unchanged).
        let generation = page.results.generation();
        page.set_query("   ".into());
        page.submit(&cx);
        assert_eq!(page.results.generation(), generation);
    }
}
