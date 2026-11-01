//! Universal Component Protocol (UCP) traits for quoin.
//!
//! This crate defines framework-agnostic adapter traits for complex UI
//! components like virtual lists, data tables, and rich text. Framework
//! adapters (e.g., `quoin-ui-gpui`) provide concrete implementations.

pub mod clipboard;

/// Sort direction for data tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Column is not sorted.
    #[default]
    None,
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Marker trait for virtual list adapters.
pub trait VirtualListAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for data table adapters.
pub trait TableAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for text input adapters.
pub trait TextInputAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for button adapters.
pub trait ButtonAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for dropdown menu adapters.
pub trait DropdownMenuAdapter: Default + Clone + Send + Sync + 'static {}

/// Marker trait for tab bar adapters.
pub trait TabBarAdapter: Default + Clone + Send + Sync + 'static {}

/// Component size variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ComponentSize {
    /// Compact density.
    Small,
    /// Default density.
    #[default]
    Medium,
    /// Roomy density.
    Large,
}

/// Button variant descriptor.
#[derive(Clone, Debug, Default)]
pub struct ButtonVariant {
    /// High-emphasis (filled) styling.
    pub primary: bool,
    /// Transparent/borderless styling.
    pub ghost: bool,
    /// Dangerous-action styling.
    pub destructive: bool,
    /// Density of the button.
    pub size: ComponentSize,
}

/// Theme color tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeToken {
    /// Primary brand color.
    Primary,
    /// Secondary brand color.
    Secondary,
    /// Page background.
    Background,
    /// Default text color.
    Foreground,
    /// De-emphasized surface color.
    Muted,
    /// De-emphasized text color.
    MutedForeground,
    /// Accent highlight color.
    Accent,
    /// Informational color.
    Info,
    /// Warning color.
    Warning,
    /// Destructive-action color.
    Danger,
    /// Border color.
    Border,
    /// Input surface color.
    Input,
    /// Focus-ring color.
    Ring,
}

/// Framework-specific theme resolution.
pub trait QuoinTheme {
    /// The framework's native color type.
    type Color;

    /// Resolves a theme token to the framework color.
    fn resolve(token: ThemeToken) -> Self::Color;

    /// Resolves a theme token with the given opacity applied.
    fn resolve_with_opacity(token: ThemeToken, opacity: f32) -> Self::Color;
}
