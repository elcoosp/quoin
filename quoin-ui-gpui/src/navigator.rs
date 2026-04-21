//! GPUI implementation of the Navigator trait.

use quoin_ui::navigator::Navigator;
use std::sync::Arc;

/// GPUI navigator that delegates to application-provided callbacks.
///
/// Since GPUI does not have a built-in router, this struct allows the
/// application to inject its own navigation logic.
///
/// # Example
///
/// ```rust,ignore
/// let nav = GpuiNavigator::new(
///     |path| println!("push: {}", path),
///     |path| println!("replace: {}", path),
///     || println!("back"),
///     || println!("forward"),
///     || "/".to_string(),
/// );
/// nav.push("/settings");
/// ```
#[derive(Clone)]
pub struct GpuiNavigator {
    push_fn: Arc<dyn Fn(&str) + Send + Sync>,
    replace_fn: Arc<dyn Fn(&str) + Send + Sync>,
    back_fn: Arc<dyn Fn() + Send + Sync>,
    forward_fn: Arc<dyn Fn() + Send + Sync>,
    current_path_fn: Arc<dyn Fn() -> String + Send + Sync>,
}

impl GpuiNavigator {
    /// Create a new navigator with the given callbacks.
    #[must_use]
    pub fn new(
        push_fn: impl Fn(&str) + Send + Sync + 'static,
        replace_fn: impl Fn(&str) + Send + Sync + 'static,
        back_fn: impl Fn() + Send + Sync + 'static,
        forward_fn: impl Fn() + Send + Sync + 'static,
        current_path_fn: impl Fn() -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            push_fn: Arc::new(push_fn),
            replace_fn: Arc::new(replace_fn),
            back_fn: Arc::new(back_fn),
            forward_fn: Arc::new(forward_fn),
            current_path_fn: Arc::new(current_path_fn),
        }
    }

    /// Create a no-op navigator that does nothing.
    /// Useful for components that don't need navigation.
    #[must_use]
    pub fn noop() -> Self {
        Self::new(
            |_| {},
            |_| {},
            || {},
            || {},
            || String::new(),
        )
    }
}

impl Navigator for GpuiNavigator {
    fn push(&self, path: &str) {
        (self.push_fn)(path);
    }

    fn replace(&self, path: &str) {
        (self.replace_fn)(path);
    }

    fn back(&self) {
        (self.back_fn)();
    }

    fn forward(&self) {
        (self.forward_fn)();
    }

    fn current_path(&self) -> String {
        (self.current_path_fn)()
    }
}

impl Default for GpuiNavigator {
    fn default() -> Self {
        Self::noop()
    }
}

impl std::fmt::Debug for GpuiNavigator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuiNavigator").finish()
    }
}
