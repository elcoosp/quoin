//! Cross-platform navigation abstraction.
//!
//! Provides a [`Navigator`] trait that framework adapters implement
//! to manage browser-like routing and history.

/// A cross-platform navigation interface.
///
/// Implementations are provided by framework adapter crates:
/// - `quoin-ui-gpui` uses callback-based navigation (GPUI has no built-in router)
/// - Web adapters (Leptos/Dioxus) wrap their framework's router
///
/// # Example
///
/// ```rust,ignore
/// fn go_home(nav: &Navigator) {
///     nav.push("/home");
/// }
/// ```
pub trait Navigator: Clone + Send + Sync + 'static {
    /// Push a new route onto the history stack.
    fn push(&self, path: &str);

    /// Replace the current route without adding to history.
    fn replace(&self, path: &str);

    /// Navigate back in history.
    fn back(&self);

    /// Navigate forward in history.
    fn forward(&self);

    /// Returns the current path.
    fn current_path(&self) -> String;
}

/// A no-op navigator for testing or contexts without routing.
#[derive(Clone, Default, Debug)]
pub struct StubNavigator;

impl Navigator for StubNavigator {
    fn push(&self, _path: &str) {}
    fn replace(&self, _path: &str) {}
    fn back(&self) {}
    fn forward(&self) {}
    fn current_path(&self) -> String {
        String::new()
    }
}
