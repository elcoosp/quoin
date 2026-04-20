//! Universal Component Protocol (UCP) traits for quoin.
//!
//! This crate defines framework-agnostic adapter traits for complex UI
//! components like virtual lists, data tables, and rich text. Framework
//! adapters (e.g., `quoin-ui-gpui`) provide concrete implementations.

/// Sort direction for data tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
    None,
}

/// Marker trait for virtual list adapters.
pub trait VirtualListAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for data table adapters.
pub trait TableAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for text input adapters.
pub trait TextInputAdapter: Default + Clone + Send + Sync + 'static {}

/// Theme color tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeToken {
    Primary,
    Secondary,
    Background,
    Foreground,
    Muted,
    MutedForeground,
    Accent,
    Info,
    Warning,
    Danger,
    Border,
    Input,
    Ring,
}

/// Framework-specific theme resolution.
pub trait QuoinTheme {
    type Color;

    fn resolve(token: ThemeToken) -> Self::Color;
    fn resolve_with_opacity(token: ThemeToken, opacity: f32) -> Self::Color;
}
