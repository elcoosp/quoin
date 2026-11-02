//! Universal Component Protocol (UCP) traits for quoin.
//!
//! This crate defines framework-agnostic adapter traits for complex UI
//! components like virtual lists, data tables, and rich text. Framework
//! adapters (e.g., `quoin-ui-gpui`) provide concrete implementations.

pub mod clipboard;
pub mod node;

pub use node::{
    ButtonNode, ButtonSize, ClipboardNode, ColumnSpec, CustomNode, DivBuilder, DivNode, IconNode,
    InputNode, InputValues, MenuItem, MenuProvider, Node, NodeButtonVariant, NodeChild, NodeEnv,
    NodeEventCtx, NodeFactory, NodeFontWeight, NodeHandler, NodeKeySpec, NodeStyle,
    ScrollRestorationNode, StyleCmd, StyledTextNode, TabBarNode, TabSpec, TableNode, TextNode,
    VirtualListNode, button, clipboard, custom, data_table, input, scroll_restoration, tab_bar,
    virtual_list,
};

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
    /// Success color.
    Success,
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

/// A color specification for framework-agnostic styled text: a theme token,
/// the same token with an opacity multiplier, or a raw sRGB hex value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextColor {
    /// A theme token.
    Token(ThemeToken),
    /// A theme token with the given opacity multiplier applied.
    TokenWithOpacity(ThemeToken, f32),
    /// A raw `0xRRGGBB` color.
    Hex(u32),
}

/// One styled run of a `styled_text` element: a number of characters plus
/// optional foreground/background colors. Runs partition the text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyledRun {
    /// Number of characters covered by this run.
    pub len: usize,
    /// Foreground color, when restyled.
    pub color: Option<TextColor>,
    /// Background color, when highlighted.
    pub bg: Option<TextColor>,
}

impl StyledRun {
    /// A run with only a foreground color.
    #[must_use]
    pub const fn fg(len: usize, color: TextColor) -> Self {
        Self {
            len,
            color: Some(color),
            bg: None,
        }
    }

    /// A run with foreground and background colors.
    #[must_use]
    pub const fn fg_bg(len: usize, color: TextColor, bg: TextColor) -> Self {
        Self {
            len,
            color: Some(color),
            bg: Some(bg),
        }
    }
}

/// Opens a URL with the platform handler (browser, mail client, …).
///
/// Implemented per adapter; the GPUI adapter delegates to the window's
/// `open_url`. Provided to framework-agnostic libraries through the
/// context/global mechanism.
pub trait Shell: Send + Sync + 'static {
    /// Hands the URL to the operating system.
    fn open_url(&self, url: &str);
}

/// Reports the host window viewport size in logical pixels, so framework-
/// agnostic panels can adapt to the window (devtools panel clamping, …).
///
/// Adapters that cannot know their viewport return values larger than any
/// panel, which makes size clamping a no-op.
pub trait ViewportInfo: Send + Sync + 'static {
    /// `(width, height)` of the host viewport in logical pixels.
    fn viewport_size(&self) -> (f32, f32);
}
