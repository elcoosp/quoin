//! GPUI backend for the Universal Component Protocol.
//!
//! Provides GPUI-specific implementations of `quoin-ui` adapter traits.

use gpui::{Hsla, Rgba};
use quoin_ui::{QuoinTheme, TableAdapter, TextInputAdapter, ThemeToken, VirtualListAdapter};

// -----------------------------------------------------------------------------
// Theme Resolution
// -----------------------------------------------------------------------------

pub struct GpuiTheme;

impl QuoinTheme for GpuiTheme {
    type Color = Hsla;

    fn resolve(token: ThemeToken) -> Self::Color {
        match token {
            ThemeToken::Primary => Hsla::from(Rgba { r: 0.23, g: 0.46, b: 0.97, a: 1.0 }),
            ThemeToken::Secondary => Hsla::from(Rgba { r: 0.42, g: 0.45, b: 0.50, a: 1.0 }),
            ThemeToken::Background => Hsla::from(Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            ThemeToken::Foreground => Hsla::from(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
            ThemeToken::Muted => Hsla::from(Rgba { r: 0.96, g: 0.96, b: 0.96, a: 1.0 }),
            ThemeToken::MutedForeground => Hsla::from(Rgba { r: 0.45, g: 0.45, b: 0.45, a: 1.0 }),
            ThemeToken::Accent | ThemeToken::Info => Hsla::from(Rgba { r: 0.23, g: 0.46, b: 0.97, a: 1.0 }),
            ThemeToken::Warning => Hsla::from(Rgba { r: 0.98, g: 0.72, b: 0.18, a: 1.0 }),
            ThemeToken::Danger => Hsla::from(Rgba { r: 0.94, g: 0.27, b: 0.27, a: 1.0 }),
            ThemeToken::Border => Hsla::from(Rgba { r: 0.90, g: 0.90, b: 0.90, a: 1.0 }),
            ThemeToken::Input => Hsla::from(Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            ThemeToken::Ring => Hsla::from(Rgba { r: 0.23, g: 0.46, b: 0.97, a: 0.4 }),
        }
    }

    fn resolve_with_opacity(token: ThemeToken, opacity: f32) -> Self::Color {
        let mut color = Self::resolve(token);
        color.a = (color.a * opacity).clamp(0.0, 1.0);
        color
    }
}

/// Convenience function to resolve a color token from a string name.
pub fn resolve_color(name: &str) -> Hsla {
    match name {
        "primary" => GpuiTheme::resolve(ThemeToken::Primary),
        "secondary" => GpuiTheme::resolve(ThemeToken::Secondary),
        "background" => GpuiTheme::resolve(ThemeToken::Background),
        "foreground" => GpuiTheme::resolve(ThemeToken::Foreground),
        "muted" => GpuiTheme::resolve(ThemeToken::Muted),
        "muted-foreground" => GpuiTheme::resolve(ThemeToken::MutedForeground),
        "accent" | "info" => GpuiTheme::resolve(ThemeToken::Info),
        "warning" => GpuiTheme::resolve(ThemeToken::Warning),
        "danger" => GpuiTheme::resolve(ThemeToken::Danger),
        "border" => GpuiTheme::resolve(ThemeToken::Border),
        "input" => GpuiTheme::resolve(ThemeToken::Input),
        "ring" => GpuiTheme::resolve(ThemeToken::Ring),
        _ => gpui::black(),
    }
}

// -----------------------------------------------------------------------------
// Adapters
// -----------------------------------------------------------------------------

/// GPUI virtual list adapter.
/// Holds a scroll handle for the virtual list.
#[derive(Default, Clone)]
pub struct GpuiVirtualListAdapter {
    // When gpui-component is available, this will hold VirtualListScrollHandle
    // pub scroll_handle: gpui_component::VirtualListScrollHandle,
}

impl VirtualListAdapter for GpuiVirtualListAdapter {}

/// GPUI table adapter.
#[derive(Default, Clone)]
pub struct GpuiTableAdapter {
    pub striped: bool,
}

impl TableAdapter for GpuiTableAdapter {}

/// GPUI text input adapter.
#[derive(Default, Clone)]
pub struct GpuiTextInputAdapter;

impl TextInputAdapter for GpuiTextInputAdapter {}

// -----------------------------------------------------------------------------
// Icon Mapping
// -----------------------------------------------------------------------------

/// Maps quoin icon name strings to GPUI icon names (when gpui-component is available).
pub fn resolve_icon(name: &str) -> &'static str {
    // Placeholder: in the future this will map to gpui_component::IconName
    // For now, we return a static string by leaking memory (safe for limited use)
    // or we can use a match that returns known static strings.
    match name {
        "info" => "info",
        "calendar" => "calendar",
        "folder" => "folder",
        "inbox" => "inbox",
        "settings" => "settings",
        "search" => "search",
        "close" => "close",
        "trash" => "trash",
        "refresh" => "refresh",
        "play" => "play",
        "layout" => "layout",
        "file" => "file",
        "map" => "map",
        "copy" => "copy",
        "check" => "check",
        "chevron-down" => "chevron-down",
        "chevron-right" => "chevron-right",
        _ => "info",
    }
}
