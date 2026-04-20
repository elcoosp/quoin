//! GPUI backend for the Universal Component Protocol.
//!
//! Provides GPUI-specific implementations of `quoin-ui` adapter traits
//! and render functions for UCP components.

use gpui::{Hsla, IntoElement, ParentElement, Rgba, Styled};
use quoin_ui::{
    ButtonAdapter, ButtonVariant, ComponentSize, DropdownMenuAdapter, QuoinTheme, TabBarAdapter,
    TableAdapter, TextInputAdapter, ThemeToken, VirtualListAdapter, clipboard::Clipboard,
};
use std::sync::Arc;

// -----------------------------------------------------------------------------
// Theme Resolution
// -----------------------------------------------------------------------------

pub struct GpuiTheme;

impl QuoinTheme for GpuiTheme {
    type Color = Hsla;

    fn resolve(token: ThemeToken) -> Self::Color {
        match token {
            ThemeToken::Primary => Hsla::from(Rgba {
                r: 0.23,
                g: 0.46,
                b: 0.97,
                a: 1.0,
            }),
            ThemeToken::Secondary => Hsla::from(Rgba {
                r: 0.42,
                g: 0.45,
                b: 0.50,
                a: 1.0,
            }),
            ThemeToken::Background => Hsla::from(Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            ThemeToken::Foreground => Hsla::from(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            ThemeToken::Muted => Hsla::from(Rgba {
                r: 0.96,
                g: 0.96,
                b: 0.96,
                a: 1.0,
            }),
            ThemeToken::MutedForeground => Hsla::from(Rgba {
                r: 0.45,
                g: 0.45,
                b: 0.45,
                a: 1.0,
            }),
            ThemeToken::Accent | ThemeToken::Info => Hsla::from(Rgba {
                r: 0.23,
                g: 0.46,
                b: 0.97,
                a: 1.0,
            }),
            ThemeToken::Warning => Hsla::from(Rgba {
                r: 0.98,
                g: 0.72,
                b: 0.18,
                a: 1.0,
            }),
            ThemeToken::Danger => Hsla::from(Rgba {
                r: 0.94,
                g: 0.27,
                b: 0.27,
                a: 1.0,
            }),
            ThemeToken::Border => Hsla::from(Rgba {
                r: 0.90,
                g: 0.90,
                b: 0.90,
                a: 1.0,
            }),
            ThemeToken::Input => Hsla::from(Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            }),
            ThemeToken::Ring => Hsla::from(Rgba {
                r: 0.23,
                g: 0.46,
                b: 0.97,
                a: 0.4,
            }),
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
// Button
// -----------------------------------------------------------------------------

/// Render a button using basic GPUI primitives.
///
/// Returns a `gpui::Div` so that callers can chain event handlers
/// (e.g., `.on_mouse_down()`) after styling.
pub fn render_button(label: Option<String>, variant: ButtonVariant) -> gpui::Div {
    let padding = match variant.size {
        ComponentSize::Small => gpui::px(4.0),
        ComponentSize::Medium => gpui::px(8.0),
        ComponentSize::Large => gpui::px(12.0),
    };

    let mut el = gpui::div()
        .cursor_pointer()
        .rounded(gpui::px(6.0))
        .px(padding)
        .py(padding)
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white());

    if variant.primary {
        el = el.bg(gpui::rgb(0x2563eb));
    } else if variant.destructive {
        el = el.bg(gpui::rgb(0xdc2626));
    } else if variant.ghost {
        // Ghost: no background color applied
    } else {
        el = el.bg(gpui::rgb(0x4e4e4e));
    }

    if let Some(label) = label {
        el = el.child(label);
    }

    el
}

// -----------------------------------------------------------------------------
// Text Input
// -----------------------------------------------------------------------------

/// Render a text input placeholder using basic GPUI primitives.
pub fn render_input(placeholder: Option<String>, value: String) -> gpui::Div {
    let display = if value.is_empty() {
        placeholder.unwrap_or_default()
    } else {
        value
    };

    gpui::div()
        .rounded(gpui::px(6.0))
        .px(gpui::px(12.0))
        .py(gpui::px(8.0))
        .bg(gpui::rgb(0xffffff))
        .text_color(gpui::rgb(0x111827))
        .child(display)
}

// -----------------------------------------------------------------------------
// Dropdown Menu (placeholder)
// -----------------------------------------------------------------------------

/// Render a dropdown menu trigger area.
pub fn render_dropdown_trigger(label: String) -> gpui::Div {
    gpui::div()
        .cursor_pointer()
        .rounded(gpui::px(6.0))
        .px(gpui::px(12.0))
        .py(gpui::px(8.0))
        .bg(gpui::rgb(0x4e4e4e))
        .text_color(gpui::white())
        .child(label)
}

// -----------------------------------------------------------------------------
// Tab Bar
// -----------------------------------------------------------------------------

/// Render a tab bar with the given tab labels.
pub fn render_tab_bar(tabs: &[String], active_index: usize) -> gpui::Div {
    let tab_elements: Vec<gpui::AnyElement> = tabs
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let mut el = gpui::div()
                .px(gpui::px(16.0))
                .py(gpui::px(8.0))
                .cursor_pointer()
                .child(label.clone());

            if i == active_index {
                el = el.text_color(gpui::white());
            } else {
                el = el.text_color(gpui::rgb(0x9ca3af));
            }

            el.into_any_element()
        })
        .collect();

    gpui::div().flex().children(tab_elements)
}

// -----------------------------------------------------------------------------
// Data Table (placeholder)
// -----------------------------------------------------------------------------

/// Render a simple table header row.
pub fn render_table_header(columns: &[String]) -> gpui::Div {
    let header_cells: Vec<gpui::AnyElement> = columns
        .iter()
        .map(|col| {
            gpui::div()
                .px(gpui::px(12.0))
                .py(gpui::px(8.0))
                .text_color(gpui::rgb(0x6b7280))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(col.clone())
                .into_any_element()
        })
        .collect();

    gpui::div().flex().children(header_cells)
}

// -----------------------------------------------------------------------------
// Virtual List (placeholder)
// -----------------------------------------------------------------------------

/// Render a scrollable container for virtualized lists.
pub fn render_virtual_list_container() -> gpui::Div {
    gpui::div().size_full()
}

// -----------------------------------------------------------------------------
// Adapters
// -----------------------------------------------------------------------------

#[derive(Default, Clone)]
pub struct GpuiVirtualListAdapter;
impl VirtualListAdapter for GpuiVirtualListAdapter {}

#[derive(Default, Clone)]
pub struct GpuiTableAdapter {
    pub striped: bool,
}
impl TableAdapter for GpuiTableAdapter {}

#[derive(Default, Clone)]
pub struct GpuiTextInputAdapter;
impl TextInputAdapter for GpuiTextInputAdapter {}

#[derive(Default, Clone)]
pub struct GpuiButtonAdapter;
impl ButtonAdapter for GpuiButtonAdapter {}

#[derive(Default, Clone)]
pub struct GpuiDropdownMenuAdapter;
impl DropdownMenuAdapter for GpuiDropdownMenuAdapter {}

#[derive(Default, Clone)]
pub struct GpuiTabBarAdapter;
impl TabBarAdapter for GpuiTabBarAdapter {}

// -----------------------------------------------------------------------------
// Clipboard
// -----------------------------------------------------------------------------

/// GPUI clipboard implementation using callback closures.
pub struct GpuiClipboard {
    write_fn: Arc<dyn Fn(&str) + Send + Sync>,
    read_fn: Arc<dyn Fn() -> Option<String> + Send + Sync>,
}

impl GpuiClipboard {
    pub fn new(
        write_fn: impl Fn(&str) + Send + Sync + 'static,
        read_fn: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            write_fn: Arc::new(write_fn),
            read_fn: Arc::new(read_fn),
        }
    }
}

impl Clipboard for GpuiClipboard {
    fn write_text(&self, text: &str) {
        (self.write_fn)(text);
    }

    fn read_text(&self) -> Option<String> {
        (self.read_fn)()
    }
}

// -----------------------------------------------------------------------------
// Icon Mapping
// -----------------------------------------------------------------------------

/// Maps quoin icon name strings to GPUI icon names.
pub fn resolve_icon(name: &str) -> &'static str {
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
