//! GPUI backend for the Universal Component Protocol.
//!
//! Provides GPUI-specific implementations of `quoin-ui` adapter traits
//! and render functions for UCP components.

use gpui::{Entity, Hsla, IntoElement, ParentElement, Rgba, Styled, Subscription};
use quoin_ui::{
    ButtonAdapter, ButtonVariant, ComponentSize, DropdownMenuAdapter, QuoinTheme, TabBarAdapter,
    TableAdapter, TextInputAdapter, ThemeToken, VirtualListAdapter, clipboard::Clipboard,
};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export GPUI component types for use in generated code
pub use gpui_component::input::{Input, InputState};

// -----------------------------------------------------------------------------
// Input State Management
// -----------------------------------------------------------------------------

/// Manages input state entities and their change subscriptions.
///
/// Add this as a field (named `_quoin_inputs`) in any component that uses
/// `input()` elements inside `quoin_render!`. The macro-generated code
/// handles initialization and two-way signal binding automatically.
pub struct QuoinInputManager {
    states: HashMap<String, Entity<InputState>>,
    subs: HashMap<String, Subscription>,
}

impl QuoinInputManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            subs: HashMap::new(),
        }
    }

    /// Whether an input with `id` is registered.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.states.contains_key(id)
    }

    /// Registers an input entity (and its change subscription) under `id`.
    pub fn insert(&mut self, id: String, entity: Entity<InputState>, sub: Subscription) {
        self.states.insert(id.clone(), entity);
        self.subs.insert(id, sub);
    }

    /// The registered input entity for `id`, if any.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Entity<InputState>> {
        self.states.get(id)
    }
}

impl Default for QuoinInputManager {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Theme Resolution
// -----------------------------------------------------------------------------

/// Light shadcn-style theme resolved onto GPUI `Hsla` colors.
pub struct GpuiTheme;

impl QuoinTheme for GpuiTheme {
    type Color = Hsla;

    fn resolve(token: ThemeToken) -> Self::Color {
        /// One palette entry: the tokens sharing this rgba value.
        struct Entry {
            tokens: &'static [ThemeToken],
            rgba: [f32; 4],
        }

        const fn entry(tokens: &'static [ThemeToken], rgba: [f32; 4]) -> Entry {
            Entry { tokens, rgba }
        }

        use ThemeToken::{
            Accent, Background, Border, Danger, Foreground, Info, Input, Muted, MutedForeground,
            Primary, Ring, Secondary, Warning,
        };

        // The light shadcn palette; tokens sharing a color are grouped.
        let palette: [Entry; 10] = [
            entry(&[Primary, Accent, Info], [0.23, 0.46, 0.97, 1.0]),
            entry(&[Secondary], [0.42, 0.45, 0.50, 1.0]),
            entry(&[Background, Input], [1.0, 1.0, 1.0, 1.0]),
            entry(&[Foreground], [0.0, 0.0, 0.0, 1.0]),
            entry(&[Muted], [0.96, 0.96, 0.96, 1.0]),
            entry(&[MutedForeground], [0.45, 0.45, 0.45, 1.0]),
            entry(&[Warning], [0.98, 0.72, 0.18, 1.0]),
            entry(&[Danger], [0.94, 0.27, 0.27, 1.0]),
            entry(&[Border], [0.90, 0.90, 0.90, 1.0]),
            entry(&[Ring], [0.23, 0.46, 0.97, 0.4]),
        ];

        let rgba = palette
            .iter()
            .find(|e| e.tokens.contains(&token))
            .map_or([0.0, 0.0, 0.0, 1.0], |e| e.rgba);

        Hsla::from(Rgba {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        })
    }

    fn resolve_with_opacity(token: ThemeToken, opacity: f32) -> Self::Color {
        let mut color = Self::resolve(token);
        color.a = (color.a * opacity).clamp(0.0, 1.0);
        color
    }
}

/// Resolves a color token from its kebab-case name (e.g. `"muted-foreground"`).
///
/// Unknown names resolve to black.
#[must_use]
pub fn resolve_color(name: &str) -> Hsla {
    let token = match name {
        "primary" => ThemeToken::Primary,
        "secondary" => ThemeToken::Secondary,
        "background" => ThemeToken::Background,
        "foreground" => ThemeToken::Foreground,
        "muted" => ThemeToken::Muted,
        "muted-foreground" => ThemeToken::MutedForeground,
        "accent" | "info" => ThemeToken::Accent,
        "warning" => ThemeToken::Warning,
        "danger" => ThemeToken::Danger,
        "border" => ThemeToken::Border,
        "input" => ThemeToken::Input,
        "ring" => ThemeToken::Ring,
        _ => return gpui::black(),
    };
    GpuiTheme::resolve(token)
}

// -----------------------------------------------------------------------------
// Button
// -----------------------------------------------------------------------------

#[must_use]
/// Renders a styled button div for the given label and variant.
pub fn render_button(label: Option<String>, variant: &ButtonVariant) -> gpui::Div {
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
        el = el.bg(gpui::rgb(0x25_63eb));
    } else if variant.destructive {
        el = el.bg(gpui::rgb(0xdc_2626));
    } else if variant.ghost {
        // Ghost: no background
    } else {
        el = el.bg(gpui::rgb(0x4e_4e4e));
    }

    if let Some(label) = label {
        el = el.child(label);
    }

    el
}

// -----------------------------------------------------------------------------
// Text Input
// -----------------------------------------------------------------------------

#[must_use]
/// Renders a static input representation showing `value` (or the placeholder).
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
        .bg(gpui::rgb(0xff_ffff))
        .text_color(gpui::rgb(0x11_1827))
        .child(display)
}

// -----------------------------------------------------------------------------
// Dropdown Menu
// -----------------------------------------------------------------------------

#[must_use]
/// Renders a dropdown trigger div with the given label.
pub fn render_dropdown_trigger(label: String) -> gpui::Div {
    gpui::div()
        .cursor_pointer()
        .rounded(gpui::px(6.0))
        .px(gpui::px(12.0))
        .py(gpui::px(8.0))
        .bg(gpui::rgb(0x4e_4e4e))
        .text_color(gpui::white())
        .child(label)
}

// -----------------------------------------------------------------------------
// Tab Bar
// -----------------------------------------------------------------------------

#[must_use]
/// Renders a horizontal tab bar, highlighting `active_index`.
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
                el = el.text_color(gpui::rgb(0x9c_a3af));
            }
            el.into_any_element()
        })
        .collect();
    gpui::div().flex().children(tab_elements)
}

// -----------------------------------------------------------------------------
// Data Table
// -----------------------------------------------------------------------------

#[must_use]
/// Renders a table header row for the given column names.
pub fn render_table_header(columns: &[String]) -> gpui::Div {
    let header_cells: Vec<gpui::AnyElement> = columns
        .iter()
        .map(|col| {
            gpui::div()
                .px(gpui::px(12.0))
                .py(gpui::px(8.0))
                .text_color(gpui::rgb(0x6b_7280))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(col.clone())
                .into_any_element()
        })
        .collect();
    gpui::div().flex().children(header_cells)
}

// -----------------------------------------------------------------------------
// Virtual List
// -----------------------------------------------------------------------------

#[must_use]
/// Renders the scrollable container that hosts a virtual list.
pub fn render_virtual_list_container() -> gpui::Div {
    gpui::div().size_full()
}

// -----------------------------------------------------------------------------
// Adapters
// -----------------------------------------------------------------------------

// Each adapter type pairs the UCP marker trait with an inherent `render`
// method, so callers get a discoverable, type-checked API while remaining
// free to call the free functions directly.

/// Virtual-list adapter backed by a scrollable GPUI div.
#[derive(Default, Clone)]
pub struct GpuiVirtualListAdapter;
impl VirtualListAdapter for GpuiVirtualListAdapter {}

impl GpuiVirtualListAdapter {
    /// Render the scrollable container that hosts a virtual list.
    #[must_use]
    pub fn render(&self) -> gpui::Div {
        render_virtual_list_container()
    }
}

/// Data-table adapter rendering a styled GPUI header row.
#[derive(Default, Clone)]
pub struct GpuiTableAdapter {
    /// Whether alternating rows should be shaded.
    pub striped: bool,
}
impl TableAdapter for GpuiTableAdapter {}

impl GpuiTableAdapter {
    /// Render a table header row for the given column names.
    #[must_use]
    pub fn render_header(&self, columns: &[String]) -> gpui::Div {
        render_table_header(columns)
    }
}

/// Text-input adapter rendering a static GPUI representation.
#[derive(Default, Clone)]
pub struct GpuiTextInputAdapter;
impl TextInputAdapter for GpuiTextInputAdapter {}

impl GpuiTextInputAdapter {
    /// Render a (static) input representation. For a fully interactive,
    /// two-way bound input, use the `input()` element inside `quoin_render!`,
    /// which wires a real `gpui_component::Input` through `QuoinInputManager`.
    #[must_use]
    pub fn render(&self, placeholder: Option<String>, value: String) -> gpui::Div {
        render_input(placeholder, value)
    }
}

/// Button adapter rendering styled GPUI button divs.
#[derive(Default, Clone)]
pub struct GpuiButtonAdapter;
impl ButtonAdapter for GpuiButtonAdapter {}

impl GpuiButtonAdapter {
    /// Render a styled button with the given label and variant.
    #[must_use]
    pub fn render(&self, label: Option<String>, variant: &ButtonVariant) -> gpui::Div {
        render_button(label, variant)
    }
}

/// Dropdown-menu adapter rendering the GPUI trigger.
#[derive(Default, Clone)]
pub struct GpuiDropdownMenuAdapter;
impl DropdownMenuAdapter for GpuiDropdownMenuAdapter {}

impl GpuiDropdownMenuAdapter {
    /// Render a dropdown menu trigger with the given label.
    #[must_use]
    pub fn render_trigger(&self, label: String) -> gpui::Div {
        render_dropdown_trigger(label)
    }
}

/// Tab-bar adapter rendering a GPUI tab strip.
#[derive(Default, Clone)]
pub struct GpuiTabBarAdapter;
impl TabBarAdapter for GpuiTabBarAdapter {}

impl GpuiTabBarAdapter {
    /// Render a tab bar with the given labels and the active index.
    #[must_use]
    pub fn render(&self, tabs: &[String], active_index: usize) -> gpui::Div {
        render_tab_bar(tabs, active_index)
    }
}

// -----------------------------------------------------------------------------
// Clipboard
// -----------------------------------------------------------------------------

/// GPUI clipboard backed by injectable read/write closures.
pub struct GpuiClipboard {
    write_fn: Arc<dyn Fn(&str) + Send + Sync>,
    read_fn: Arc<dyn Fn() -> Option<String> + Send + Sync>,
}

impl GpuiClipboard {
    /// Creates a clipboard from platform read/write closures (usually
    /// bound to GPUI's clipboard API in the adapter's app bootstrap).
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

/// Icon names the GPUI backend can render (checked by [`resolve_icon`]).
static KNOWN_ICONS: &[&str] = &[
    "info",
    "calendar",
    "folder",
    "inbox",
    "settings",
    "search",
    "close",
    "trash",
    "refresh",
    "play",
    "layout",
    "file",
    "map",
    "copy",
    "check",
    "chevron-down",
    "chevron-right",
];

/// Maps a quoin icon name to the GPUI icon name.
///
/// Every quoin icon name is already a valid GPUI icon name, so known names
/// pass through unchanged; unknown names fall back to `"info"`.
#[must_use]
pub fn resolve_icon(name: &str) -> &'static str {
    if KNOWN_ICONS.contains(&name) {
        // Every known icon maps to itself; the `'static` names live in
        // `KNOWN_ICONS`, so we can return the canonical literal.
        match name {
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
    } else {
        "info"
    }
}
