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

pub mod node_renderer;
pub use node_renderer::{
    GpuiCustomElement, GpuiInputValues, NodeElement, NodeKeyBinding, NodeRenderer,
    QuoinNodeTableDelegate, bind_node_keys, custom_element, resolve_node_color, resolve_token_live,
    shared_scroll_handle,
};

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

// -----------------------------------------------------------------------------
// Styled Text (UCP)
// -----------------------------------------------------------------------------

/// Resolves a [`TextColor`] to a GPUI color.
#[must_use]
pub fn resolve_text_color(color: quoin_ui::TextColor) -> gpui::Hsla {
    use quoin_ui::TextColor;
    match color {
        TextColor::Token(token) => GpuiTheme::resolve(token),
        TextColor::TokenWithOpacity(token, opacity) => {
            GpuiTheme::resolve_with_opacity(token, opacity)
        }
        TextColor::Hex(hex) => gpui::rgb(hex).into(),
    }
}

/// Builds a GPUI `StyledText` from framework-agnostic runs.
#[must_use]
pub fn build_styled_text(
    text: impl Into<gpui::SharedString>,
    runs: &[quoin_ui::StyledRun],
) -> gpui::StyledText {
    use gpui::TextStyle;
    use quoin_ui::StyledRun;
    let text = text.into();
    let gpui_runs: Vec<gpui::TextRun> = runs
        .iter()
        .map(|run: &StyledRun| {
            let mut style = TextStyle::default();
            if let Some(color) = run.color {
                style.color = resolve_text_color(color);
            }
            if let Some(bg) = run.bg {
                style.background_color = Some(resolve_text_color(bg));
            }
            style.to_run(run.len)
        })
        .collect();
    gpui::StyledText::new(text).with_runs(gpui_runs)
}

// -----------------------------------------------------------------------------
// Data Table State Manager
// -----------------------------------------------------------------------------

/// The shared, closure-driven `TableDelegate`: columns and renderers are
/// plain data + closures, so a single delegate type serves every
/// `data_table(...)` usage in generated code.
/// Type-erased row renderer: renders one column cell from the erased row.
pub type RowRenderer = std::sync::Arc<dyn Fn(&dyn std::any::Any) -> gpui::AnyElement + Send + Sync>;
/// Sort click handler: receives the column key and ascending flag.
pub type SortHandler = std::sync::Arc<dyn Fn(&str, bool) + Send + Sync>;

/// A [`gpui_component::table::TableDelegate`] driven entirely by closures,
/// letting framework-agnostic node tables plug arbitrary cell renderers in.
pub struct QuoinClosureTableDelegate {
    /// Column keys (in order).
    pub keys: Vec<&'static str>,
    /// Column header labels (in order).
    pub labels: Vec<&'static str>,
    /// Column fixed widths; `None` lets the table size the column.
    pub widths: Vec<Option<gpui::Pixels>>,
    /// Whether a column shows the sort toggle.
    pub sortables: Vec<bool>,
    /// Whether a column is user-resizable.
    pub resizables: Vec<bool>,
    /// Type-erased row payloads.
    pub entries: Vec<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    /// Per-column render closures over the erased row.
    pub renderers: Vec<RowRenderer>,
    /// Sort callback receiving `(column_key, ascending)`.
    pub on_sort: Option<SortHandler>,
}

impl gpui_component::table::TableDelegate for QuoinClosureTableDelegate {
    fn columns_count(&self, _: &gpui::App) -> usize {
        self.keys.len()
    }

    fn rows_count(&self, _: &gpui::App) -> usize {
        self.entries.len()
    }

    fn column(&self, col_ix: usize, _: &gpui::App) -> gpui_component::table::Column {
        let mut col = gpui_component::table::Column::new(self.keys[col_ix], self.labels[col_ix]);
        if let Some(width) = self.widths[col_ix] {
            col = col.width(width);
        }
        col = col.resizable(self.resizables[col_ix]);
        if self.sortables[col_ix] {
            col = col.sortable();
        }
        col
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: gpui_component::table::ColumnSort,
        _: &mut gpui::Window,
        _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        if let (Some(on_sort), Some(key)) = (&self.on_sort, self.keys.get(col_ix).copied()) {
            on_sort(key, sort == gpui_component::table::ColumnSort::Ascending);
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut gpui::Window,
        _: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let entry = self.entries[row_ix].clone();
        (self.renderers[col_ix])(entry.as_ref())
    }
}

/// Persists `TableState<QuoinClosureTableDelegate>` entities across renders
/// for `data_table(...)` elements, keyed by the element's `id`.
///
/// Add as a field (named `_quoin_tables`) in components that use
/// `data_table(...)` inside `quoin_render!`.
#[derive(Default)]
pub struct QuoinTableManager {
    tables:
        HashMap<String, gpui::Entity<gpui_component::table::TableState<QuoinClosureTableDelegate>>>,
}

impl QuoinTableManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a table with `id` is registered.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.tables.contains_key(id)
    }

    /// Registers a freshly created table state under `id`.
    pub fn insert(
        &mut self,
        id: &str,
        entity: gpui::Entity<gpui_component::table::TableState<QuoinClosureTableDelegate>>,
    ) {
        self.tables.insert(id.to_string(), entity);
    }

    /// The registered table state entity for `id`.
    #[must_use]
    pub fn get(
        &self,
        id: &str,
    ) -> Option<gpui::Entity<gpui_component::table::TableState<QuoinClosureTableDelegate>>> {
        self.tables.get(id).cloned()
    }
}

// -----------------------------------------------------------------------------
// Scroll Handle Manager
// -----------------------------------------------------------------------------

/// Persists scroll handles across renders for `virtual_list(...)` and
/// `tabs(...)` elements, keyed by the element's `id`.
#[derive(Default)]
pub struct QuoinScrollManager {
    virtual_lists: HashMap<String, gpui_component::VirtualListScrollHandle>,
}

impl QuoinScrollManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The (lazily created) virtual-list scroll handle for `id`.
    pub fn virtual_list(&mut self, id: &str) -> gpui_component::VirtualListScrollHandle {
        self.virtual_lists
            .entry(id.to_string())
            .or_insert_with(gpui_component::VirtualListScrollHandle::new)
            .clone()
    }
}

// -----------------------------------------------------------------------------
// Runtime Style Strings
// -----------------------------------------------------------------------------

/// Applies a Tailwind-ish style string to a GPUI div at runtime.
///
/// The compile-time transpiler in `quoin-macros-core` covers class literals
/// inside `quoin_render!`; this function covers classes that arrive at
/// runtime (framework-agnostic component APIs that take style strings, e.g.
/// `Link::active_class("font-bold")`). Supported: flex layout, gaps,
/// padding/margin, sizes, text size/colors/weights, backgrounds, borders,
/// radius, cursor, opacity, overflow, absolute offsets.
pub fn apply_style_string<D: gpui::Styled>(mut el: D, classes: &str) -> D {
    for class in classes.split_whitespace() {
        el = apply_single_class(el, class);
    }
    el
}

fn px_value(token: &str) -> Option<f32> {
    let value = token.strip_suffix("px").unwrap_or(token);
    value.parse::<f32>().ok()
}

#[expect(clippy::too_many_lines)]
fn apply_single_class<D: gpui::Styled>(mut el: D, class: &str) -> D {
    // Arbitrary values first: `p-[13px]`, `text-[#cdd6f4]`, `w-[600px]`, …
    if let Some((name, inner)) = class.split_once('[')
        && let Some(inner) = inner.strip_suffix(']')
    {
        if let Some(hex) = inner.strip_prefix('#') {
            if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                match name {
                    "bg" => el = el.bg(gpui::rgb(rgb)),
                    "text" => el = el.text_color(gpui::rgb(rgb)),
                    "border" => el = el.border_color(gpui::rgb(rgb)),
                    _ => {}
                }
                return el;
            }
        } else if let Some(px) = px_value(inner) {
            let px = gpui::px(px);
            match name {
                "p" => el = el.p(px),
                "px" => el = el.px(px),
                "py" => el = el.py(px),
                "pl" => el = el.pl(px),
                "pr" => el = el.pr(px),
                "pt" => el = el.pt(px),
                "pb" => el = el.pb(px),
                "gap" => el = el.gap(px),
                "w" => el = el.w(px),
                "h" => el = el.h(px),
                "min-w" => el = el.min_w(px),
                "min-h" => el = el.min_h(px),
                "max-w" => el = el.max_w(px),
                "max-h" => el = el.max_h(px),
                "rounded" => el = el.rounded(px),
                "text" => el = el.text_size(px),
                "top" => el = el.top(px),
                "right" => el = el.right(px),
                "bottom" => el = el.bottom(px),
                "left" => el = el.left(px),
                _ => {}
            }
            return el;
        }
    }
    match class {
        "flex" => el = el.flex(),
        "flex-col" => el = el.flex_col(),
        "flex-row" => el = el.flex_row(),
        "flex-wrap" => el = el.flex_wrap(),
        "flex-1" => el = el.flex_1(),
        "items-center" => el = el.items_center(),
        "items-start" => el = el.items_start(),
        "items-end" => el = el.items_end(),
        "justify-between" => el = el.justify_between(),
        "justify-center" => el = el.justify_center(),
        "justify-start" => el = el.justify_start(),
        "justify-end" => el = el.justify_end(),
        "gap-0" => el = el.gap_0(),
        "gap-1" => el = el.gap(gpui::px(4.0)),
        "gap-2" => el = el.gap(gpui::px(8.0)),
        "gap-3" => el = el.gap(gpui::px(12.0)),
        "gap-4" => el = el.gap(gpui::px(16.0)),
        "gap-6" => el = el.gap(gpui::px(24.0)),
        "gap-8" => el = el.gap(gpui::px(32.0)),
        "p-0" => el = el.p_0(),
        "p-1" => el = el.p(gpui::px(4.0)),
        "p-2" => el = el.p(gpui::px(8.0)),
        "p-3" => el = el.p(gpui::px(12.0)),
        "p-4" => el = el.p(gpui::px(16.0)),
        "p-8" => el = el.p(gpui::px(32.0)),
        "px-0" => el = el.px_0(),
        "px-1" => el = el.px(gpui::px(4.0)),
        "px-2" => el = el.px(gpui::px(8.0)),
        "px-3" => el = el.px(gpui::px(12.0)),
        "px-4" => el = el.px(gpui::px(16.0)),
        "py-0" => el = el.py_0(),
        "py-1" => el = el.py(gpui::px(4.0)),
        "py-2" => el = el.py(gpui::px(8.0)),
        "py-3" => el = el.py(gpui::px(12.0)),
        "py-4" => el = el.py(gpui::px(16.0)),
        "pl-1" => el = el.pl(gpui::px(4.0)),
        "pl-2" => el = el.pl(gpui::px(8.0)),
        "pl-3" => el = el.pl(gpui::px(12.0)),
        "pr-1" => el = el.pr(gpui::px(4.0)),
        "pr-2" => el = el.pr(gpui::px(8.0)),
        "pr-3" => el = el.pr(gpui::px(12.0)),
        "pt-1" => el = el.pt(gpui::px(4.0)),
        "pt-2" => el = el.pt(gpui::px(8.0)),
        "pt-3" => el = el.pt(gpui::px(12.0)),
        "pb-1" => el = el.pb(gpui::px(4.0)),
        "pb-2" => el = el.pb(gpui::px(8.0)),
        "pb-3" => el = el.pb(gpui::px(12.0)),
        "ml-1" => el = el.ml(gpui::px(4.0)),
        "ml-2" => el = el.ml(gpui::px(8.0)),
        "ml-auto" => el = el.ml_auto(),
        "mr-1" => el = el.mr(gpui::px(4.0)),
        "mr-2" => el = el.mr(gpui::px(8.0)),
        "mt-1" => el = el.mt(gpui::px(4.0)),
        "mt-2" => el = el.mt(gpui::px(8.0)),
        "mt-3" => el = el.mt(gpui::px(12.0)),
        "mb-1" => el = el.mb(gpui::px(4.0)),
        "mb-2" => el = el.mb(gpui::px(8.0)),
        "size-full" => el = el.size_full(),
        "w-full" => el = el.w_full(),
        "h-full" => el = el.h_full(),
        "max-w-full" => el = el.max_w_full(),
        "min-w-0" => el = el.min_w_0(),
        "min-h-0" => el = el.min_h_0(),
        "overflow-hidden" => el = el.overflow_hidden(),
        "truncate" => el = el.truncate(),
        "whitespace-nowrap" => el = el.whitespace_nowrap(),
        "text-xs" => el = el.text_xs(),
        "text-sm" => el = el.text_sm(),
        "text-base" => el = el.text_base(),
        "text-lg" => el = el.text_lg(),
        "text-xl" => el = el.text_xl(),
        "text-2xl" => el = el.text_2xl(),
        "font-thin" => el = el.font_weight(gpui::FontWeight::THIN),
        "font-light" => el = el.font_weight(gpui::FontWeight::LIGHT),
        "font-normal" => el = el.font_weight(gpui::FontWeight::NORMAL),
        "font-medium" => el = el.font_weight(gpui::FontWeight::MEDIUM),
        "font-semibold" => el = el.font_weight(gpui::FontWeight::SEMIBOLD),
        "font-bold" => el = el.font_weight(gpui::FontWeight::BOLD),
        "font-mono" => el = el.font_family("monospace"),
        "text-white" => el = el.text_color(gpui::white()),
        "text-black" => el = el.text_color(gpui::black()),
        "text-primary" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Primary,
            )));
        }
        "text-foreground" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Foreground,
            )));
        }
        "text-muted-foreground" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::MutedForeground,
            )));
        }
        "text-danger" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Danger,
            )));
        }
        "text-warning" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Warning,
            )));
        }
        "text-info" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Info,
            )));
        }
        "text-success" => {
            el = el.text_color(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Accent,
            )));
        }
        "bg-white" => el = el.bg(gpui::white()),
        "bg-black" => el = el.bg(gpui::black()),
        "bg-transparent" => el = el.bg(gpui::transparent_black()),
        "bg-primary" => {
            el = el.bg(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Primary,
            )));
        }
        "bg-secondary" => {
            el = el.bg(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Secondary,
            )));
        }
        "bg-muted" => {
            el = el.bg(resolve_text_color(quoin_ui::TextColor::Token(
                quoin_ui::ThemeToken::Muted,
            )));
        }
        "border" => el = el.border_1(),
        "border-0" => el = el.border_0(),
        "border-b" => el = el.border_b_1(),
        "border-t" => el = el.border_t_1(),
        "border-l" => el = el.border_l_1(),
        "border-r" => el = el.border_r_1(),
        "rounded-none" => el = el.rounded(gpui::px(0.0)),
        "rounded-sm" => el = el.rounded(gpui::px(2.0)),
        "rounded" => el = el.rounded(gpui::px(4.0)),
        "rounded-md" => el = el.rounded(gpui::px(6.0)),
        "rounded-lg" => el = el.rounded(gpui::px(8.0)),
        "rounded-xl" => el = el.rounded(gpui::px(12.0)),
        "rounded-2xl" => el = el.rounded(gpui::px(16.0)),
        "rounded-full" => el = el.rounded(gpui::px(9999.0)),
        "rounded-tl" => el = el.rounded_tl(gpui::px(4.0)),
        "rounded-tl-md" => el = el.rounded_tl(gpui::px(6.0)),
        "rounded-tl-lg" => el = el.rounded_tl(gpui::px(8.0)),
        "cursor-pointer" => el = el.cursor_pointer(),
        "cursor-default" => el = el.cursor_default(),
        "cursor-text" => el = el.cursor_text(),
        "absolute" => el = el.absolute(),
        "relative" => el = el.relative(),
        "opacity-0" => el = el.opacity(0.0),
        "opacity-25" => el = el.opacity(0.25),
        "opacity-50" => el = el.opacity(0.5),
        "opacity-75" => el = el.opacity(0.75),
        "opacity-100" => el = el.opacity(1.0),
        "shadow-sm" => el = el.shadow_sm(),
        "shadow-md" => el = el.shadow_md(),
        "shadow-lg" => el = el.shadow_lg(),
        "top-0" => el = el.top(gpui::px(0.0)),
        "right-0" => el = el.right(gpui::px(0.0)),
        "bottom-0" => el = el.bottom(gpui::px(0.0)),
        "left-0" => el = el.left(gpui::px(0.0)),
        "bottom-3" => el = el.bottom(gpui::px(12.0)),
        "right-3" => el = el.right(gpui::px(12.0)),
        _ => {}
    }
    el
}

/// Opens URLs through GPUI's clipboard-free `open_url` path. The window is
/// captured through a closure the app bootstrap wires (see
/// `GpuiShell::new`).
pub struct GpuiShell {
    open_fn: Arc<dyn Fn(&str) + Send + Sync>,
}

impl GpuiShell {
    /// Creates a shell from a closure calling `cx.open_url(...)` with the
    /// app context available.
    pub fn new(open_fn: impl Fn(&str) + Send + Sync + 'static) -> Self {
        Self {
            open_fn: Arc::new(open_fn),
        }
    }
}

impl quoin_ui::Shell for GpuiShell {
    fn open_url(&self, url: &str) {
        (self.open_fn)(url);
    }
}

/// Binds a component's `keys { .. }` keybindings once per app. Generated
/// `component!` code exposes a `__quoin_keys()`-style function returning the
/// bindings; call this at bootstrap.
pub fn bind_component_keys(cx: &mut gpui::App, bindings: Vec<gpui::KeyBinding>) {
    cx.bind_keys(bindings);
}
