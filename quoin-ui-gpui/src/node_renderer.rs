//! Runtime renderer for the framework-agnostic [`quoin_ui::Node`] tree.
//!
//! The renderer walks a `Node` tree and materializes native GPUI elements,
//! using real `gpui-component` widgets for buttons, inputs, tab bars, data
//! tables and virtual lists. Widget state (input entities, table states,
//! scroll handles) persists across renders keyed by the node ids, so
//! rebuilding the tree every frame never loses focus, text, scroll offset
//! or table layout — matching the entity-based widgets it replaces 1:1.
//!
//! Theme tokens resolve through `gpui_component`'s live `ActiveTheme`, so
//! the rendered colors are exactly the colors the host application themed.

use gpui::{
    Action, AnyElement, App, Context, Entity, FocusHandle, FontWeight, MouseButton, MouseDownEvent,
    MouseMoveEvent, SharedString, Size, StyledText, TextStyle, div, prelude::*, px, rgb,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, Size as ComponentSize, VirtualList,
    VirtualListScrollHandle,
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    input::{Input, InputState},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    scroll::{Scrollable, ScrollableElement},
    tab::{Tab, TabBar},
    table::{Column, ColumnSort, DataTable, TableDelegate, TableState},
    v_virtual_list,
};
use log::warn;
use quoin_ui::node::{
    ButtonSize, Node, NodeButtonVariant, NodeEnv, NodeEventCtx, NodeFontWeight, StyleCmd,
};
use quoin_ui::{TextColor, ThemeToken};
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// -----------------------------------------------------------------------------
// Keybindings
// -----------------------------------------------------------------------------

// A single parameterized action carries the framework-agnostic action name
// through GPUI's action dispatch: bindings map keystrokes to `NodeAction`
// instances, and every `on_action` listener receives the instance and reads
// its name.
#[derive(Clone, PartialEq, Debug, Action)]
#[action(no_json, namespace = quoin_node)]
struct NodeAction {
    name: SharedString,
}

/// One keybinding for node action dispatch (adapter alias of the
/// framework-agnostic [`quoin_ui::NodeKeySpec`]).
pub type NodeKeyBinding = quoin_ui::NodeKeySpec;

/// Binds node action keystrokes once per app (call at bootstrap).
pub fn bind_node_keys(cx: &mut App, bindings: &[NodeKeyBinding]) {
    for binding in bindings {
        let keystroke: &'static str = Box::leak(binding.keystroke.clone().into_boxed_str());
        let action_name: &'static str = Box::leak(binding.action.clone().into_boxed_str());
        let context: &'static str = Box::leak(binding.context.clone().into_boxed_str());
        cx.bind_keys([gpui::KeyBinding::new(
            keystroke,
            NodeAction {
                name: SharedString::from(action_name),
            },
            Some(context),
        )]);
    }
}

// -----------------------------------------------------------------------------
// Live theme resolution
// -----------------------------------------------------------------------------

/// Resolves a [`ThemeToken`] against gpui-component's live application
/// theme, so colors match whatever theme the host app installed.
#[must_use]
pub fn resolve_token_live(token: ThemeToken, cx: &App) -> gpui::Hsla {
    let theme = cx.theme();
    match token {
        ThemeToken::Primary => theme.primary,
        ThemeToken::Secondary => theme.secondary,
        ThemeToken::Background => theme.background,
        ThemeToken::Foreground => theme.foreground,
        ThemeToken::Muted => theme.muted,
        ThemeToken::MutedForeground => theme.muted_foreground,
        ThemeToken::Accent => theme.accent,
        ThemeToken::Info => theme.info,
        ThemeToken::Success => theme.success,
        ThemeToken::Warning => theme.warning,
        ThemeToken::Danger => theme.danger,
        ThemeToken::Border => theme.border,
        ThemeToken::Input => theme.input,
        ThemeToken::Ring => theme.ring,
    }
}

/// Resolves a framework-agnostic [`TextColor`] against the live theme.
#[must_use]
pub fn resolve_node_color(color: TextColor, cx: &App) -> gpui::Hsla {
    match color {
        TextColor::Token(token) => resolve_token_live(token, cx),
        TextColor::TokenWithOpacity(token, opacity) => {
            let mut color = resolve_token_live(token, cx);
            color.a = (color.a * opacity).clamp(0.0, 1.0);
            color
        }
        TextColor::Hex(hex) => rgb(hex).into(),
    }
}

fn font_weight(weight: NodeFontWeight) -> FontWeight {
    match weight {
        NodeFontWeight::Thin => FontWeight::THIN,
        NodeFontWeight::Light => FontWeight::LIGHT,
        NodeFontWeight::Normal => FontWeight::NORMAL,
        NodeFontWeight::Medium => FontWeight::MEDIUM,
        NodeFontWeight::Semibold => FontWeight::SEMIBOLD,
        NodeFontWeight::Bold => FontWeight::BOLD,
    }
}

/// Maps a kebab-case icon name to the GPUI icon enum.
#[must_use]
pub fn icon_variant(name: &str) -> Option<IconName> {
    Some(match name {
        "info" => IconName::Info,
        "calendar" => IconName::Calendar,
        "folder" => IconName::Folder,
        "folder-open" => IconName::FolderOpen,
        "inbox" => IconName::Inbox,
        "settings" => IconName::Settings,
        "search" => IconName::Search,
        "close" => IconName::Close,
        "delete" | "trash" => IconName::Delete,
        "refresh" => IconName::RefreshCw,
        "play" => IconName::Play,
        "layout" => IconName::LayoutDashboard,
        "file" => IconName::File,
        "map" => IconName::Map,
        "copy" => IconName::Copy,
        "check" => IconName::Check,
        "chevron-down" => IconName::ChevronDown,
        "chevron-right" => IconName::ChevronRight,
        "square-terminal" => IconName::SquareTerminal,
        "globe" => IconName::Globe,
        _ => return None,
    })
}

fn icon_element(name: &str) -> Icon {
    let variant = icon_variant(name).unwrap_or(IconName::Info);
    Icon::new(variant)
}

// -----------------------------------------------------------------------------
// Event-time clipboard / shell
// -----------------------------------------------------------------------------

struct GpuiNodeEnv<'a> {
    app: &'a mut App,
    inputs: SharedInputMap,
}

/// The per-input adapter state shared between the renderer and the
/// event-time environment.
pub(crate) struct InputEntry {
    entity: Entity<InputState>,
    /// Current change callback (swapped on every render of the owning
    /// `input(...)` node), invoked from the entity observer at edit time.
    on_change: OnChangeSlot,
    #[expect(dead_code)]
    subscription: gpui::Subscription,
}

pub(crate) type SharedInputMap = Rc<RefCell<HashMap<String, InputEntry>>>;
/// Per-input change callback slot swapped in on every render.
pub(crate) type OnChangeSlot = Rc<RefCell<Option<Rc<dyn Fn(String)>>>>;
/// Sort click handler: receives the column key and ascending flag.
pub(crate) type SortHandler = Rc<dyn Fn(&str, bool)>;
/// Looks up the saved scroll offset for a key.
pub(crate) type SavedOffsetFn = Rc<dyn Fn(&str) -> Option<f32>>;
/// Persists a scroll offset under a key.
pub(crate) type SaveOffsetFn = Rc<dyn Fn(&str, f32)>;

impl NodeEnv for GpuiNodeEnv<'_> {
    fn write_clipboard(&mut self, text: &str) {
        self.app
            .write_to_clipboard(gpui::ClipboardItem::new_string(text.to_string()));
    }

    fn open_url(&mut self, url: &str) {
        self.app.open_url(url);
    }

    fn input_value(&self, id: &str) -> Option<String> {
        let entity = self.inputs.borrow().get(id).map(|e| e.entity.clone())?;
        Some(entity.read(self.app).value().to_string())
    }
}

fn run_handler(handler: &quoin_ui::NodeHandler, cx: &mut App, inputs: &SharedInputMap) {
    let mut env = GpuiNodeEnv {
        app: cx,
        inputs: Rc::clone(inputs),
    };
    let mut ctx = NodeEventCtx {
        env: Some(&mut env),
    };
    handler(&mut ctx);
}

// -----------------------------------------------------------------------------
// Renderer state
// -----------------------------------------------------------------------------

/// The delegate backing every `Node::Table`: columns are plain data, cells
/// are lazily rendered `Node`s.
pub struct QuoinNodeTableDelegate {
    /// Column metadata in order.
    pub columns: Vec<quoin_ui::ColumnSpec>,
    /// Current row count.
    pub row_count: usize,
    /// Builds the cell node for `(row, column)`.
    pub cell: Rc<dyn Fn(usize, usize) -> Node>,
    /// Sort callback `(column_key, ascending)`.
    pub on_sort: Option<SortHandler>,
    /// Shared input map so cell handlers resolve live input values.
    pub(crate) inputs: SharedInputMap,
}

impl TableDelegate for QuoinNodeTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.row_count
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        let spec = &self.columns[col_ix];
        let mut col = Column::new(spec.key.clone(), spec.label.clone());
        if let Some(width) = spec.width {
            col = col.width(px(width));
        }
        col = col.resizable(spec.resizable);
        if spec.sortable {
            col = col.sortable();
        }
        col
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut gpui::Window,
        _: &mut gpui::Context<TableState<Self>>,
    ) {
        if let (Some(on_sort), Some(spec)) = (&self.on_sort, self.columns.get(col_ix)) {
            on_sort(&spec.key, sort == ColumnSort::Ascending);
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let node = (self.cell)(row_ix, col_ix);
        let view = node_host_view(cx);
        let mut state = RendererState {
            inputs: Rc::clone(&self.inputs),
            ..RendererState::default()
        };
        render_node_impl(&node, &view, &mut state, window, cx)
    }
}

/// Adapter-side widget state surviving across frames.
#[derive(Default)]
struct RendererState {
    inputs: SharedInputMap,
    tables: HashMap<String, Entity<TableState<QuoinNodeTableDelegate>>>,
    virtual_list_handles: HashMap<String, VirtualListScrollHandle>,
    focus_handles: HashMap<String, FocusHandle>,
}

impl RendererState {
    fn new() -> Self {
        Self {
            inputs: Rc::new(RefCell::new(HashMap::new())),
            ..Self::default()
        }
    }
}

/// Renders [`quoin_ui::Node`] trees into GPUI elements.
///
/// Owns the adapter-side widget state that must survive across frames: input
/// entities, table state entities, virtual-list scroll handles and focus
/// handles, all keyed by stable node ids.
#[derive(Default)]
pub struct NodeRenderer {
    state: RendererState,
    keybindings_bound: bool,
}

impl NodeRenderer {
    /// Creates an empty renderer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The live value of the native input registered under `id`.
    #[must_use]
    pub fn input_value(&self, id: &str, cx: &App) -> String {
        self.state
            .inputs
            .borrow()
            .get(id)
            .map(|entry| entry.entity.read(cx).value().to_string())
            .unwrap_or_default()
    }

    /// An [`quoin_ui::InputValues`] view over this renderer's live inputs;
    /// the borrow of `cx` keeps the values readable for the duration of a
    /// render pass (mirroring when framework code read `InputState`).
    #[must_use]
    pub fn input_values<'a>(&'a self, cx: &'a App) -> GpuiInputValues<'a> {
        GpuiInputValues {
            inputs: Rc::clone(&self.state.inputs),
            cx,
        }
    }

    /// Focuses the native input registered under `id`, if present.
    pub fn focus_input(&mut self, id: &str, window: &mut gpui::Window, cx: &mut App) {
        let entity = self.state.inputs.borrow().get(id).map(|e| e.entity.clone());
        if let Some(entity) = entity {
            entity.update(cx, |state, cx| state.focus(window, cx));
        }
    }

    /// Binds the keybindings once per renderer lifetime (idempotent).
    pub fn ensure_keybindings(&mut self, cx: &mut App, bindings: &[NodeKeyBinding]) {
        if !self.keybindings_bound {
            bind_node_keys(cx, bindings);
            self.keybindings_bound = true;
        }
    }

    /// Renders a node tree into a native element.
    ///
    /// `focus` optionally names an input id to focus after building the
    /// tree. Virtual lists anchor on an internal host view entity owned by
    /// the renderer, so the caller needs no entity of its own.
    pub fn render(
        &mut self,
        node: &Node,
        focus: Option<&str>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> AnyElement {
        let view = node_host_view(cx);
        let mut state = std::mem::take(&mut self.state);
        let element = render_node_impl(node, &view, &mut state, window, cx);
        self.state = state;
        if let Some(id) = focus {
            self.focus_input(id, window, cx);
        }
        element
    }
}

/// A shared read view over the renderer's native input values.
pub struct GpuiInputValues<'a> {
    inputs: SharedInputMap,
    cx: &'a App,
}

impl quoin_ui::InputValues for GpuiInputValues<'_> {
    fn value(&self, id: &str) -> String {
        self.inputs
            .borrow()
            .get(id)
            .map(|entry| entry.entity.read(self.cx).value().to_string())
            .unwrap_or_default()
    }
}

// -----------------------------------------------------------------------------
// Shared scroll handles (page-side helper)
// -----------------------------------------------------------------------------

thread_local! {
    static SHARED_SCROLL_HANDLES: RefCell<HashMap<String, gpui::ScrollHandle>> =
        RefCell::new(HashMap::new());
}

/// Returns the process-shared GPUI scroll handle for `id`, creating it on
/// first use. Pages attach this handle to their scroll containers
/// (`.track_scroll(&handle)`) while the matching
/// [`quoin_ui::Node::ScrollRestoration`] node resolves the same handle
/// during rendering, so both observe one scroll offset.
#[must_use]
pub fn shared_scroll_handle(id: &str) -> gpui::ScrollHandle {
    SHARED_SCROLL_HANDLES.with(|handles| {
        handles
            .borrow_mut()
            .entry(id.to_string())
            .or_insert_with(gpui::ScrollHandle::new)
            .clone()
    })
}

// -----------------------------------------------------------------------------
// Custom elements (app-side escape hatch)
// -----------------------------------------------------------------------------

/// The payload for rendering framework-native application elements inside a
/// node tree. The GPUI renderer recognizes exactly this type; other
/// adapters render nothing for it (the equivalent escape hatch is
/// adapter-specific by design).
pub struct GpuiCustomElement {
    render: Rc<dyn Fn() -> AnyElement>,
}

impl GpuiCustomElement {
    /// Wraps a closure producing a native element (rebuilt per render).
    pub fn new(render: impl Fn() -> AnyElement + 'static) -> Self {
        Self {
            render: Rc::new(render),
        }
    }

    fn render(&self) -> AnyElement {
        (self.render)()
    }
}

/// Builds a [`quoin_ui::Node::Custom`] rendering a native GPUI element.
pub fn custom_element(render: impl Fn() -> AnyElement + 'static) -> quoin_ui::Node {
    quoin_ui::Node::Custom(quoin_ui::CustomNode {
        type_id: TypeId::of::<GpuiCustomElement>(),
        payload: Rc::new(GpuiCustomElement::new(render)),
    })
}

// -----------------------------------------------------------------------------
// Style application
// -----------------------------------------------------------------------------

fn apply_style_cmd<D>(mut el: D, cmd: &StyleCmd, cx: &App) -> D
where
    D: gpui::Styled + gpui::InteractiveElement,
{
    match cmd {
        StyleCmd::Flex => el = el.flex(),
        StyleCmd::FlexCol => el = el.flex_col(),
        StyleCmd::FlexRow => el = el.flex_row(),
        StyleCmd::FlexWrap => el = el.flex_wrap(),
        StyleCmd::Flex1 => el = el.flex_1(),
        StyleCmd::ItemsCenter => el = el.items_center(),
        StyleCmd::ItemsStart => el = el.items_start(),
        StyleCmd::ItemsEnd => el = el.items_end(),
        StyleCmd::JustifyCenter => el = el.justify_center(),
        StyleCmd::JustifyStart => el = el.justify_start(),
        StyleCmd::JustifyEnd => el = el.justify_end(),
        StyleCmd::JustifyBetween => el = el.justify_between(),
        StyleCmd::Gap(v) => el = el.gap(px(*v)),
        StyleCmd::Padding(v) => el = el.p(px(*v)),
        StyleCmd::PaddingX(v) => el = el.px(px(*v)),
        StyleCmd::PaddingY(v) => el = el.py(px(*v)),
        StyleCmd::PaddingLeft(v) => el = el.pl(px(*v)),
        StyleCmd::PaddingRight(v) => el = el.pr(px(*v)),
        StyleCmd::PaddingTop(v) => el = el.pt(px(*v)),
        StyleCmd::PaddingBottom(v) => el = el.pb(px(*v)),
        StyleCmd::MarginTop(v) => el = el.mt(px(*v)),
        StyleCmd::MarginBottom(v) => el = el.mb(px(*v)),
        StyleCmd::MarginLeft(v) => el = el.ml(px(*v)),
        StyleCmd::MarginRight(v) => el = el.mr(px(*v)),
        StyleCmd::MarginLeftAuto => el = el.ml_auto(),
        StyleCmd::WidthFull => el = el.w_full(),
        StyleCmd::HeightFull => el = el.h_full(),
        StyleCmd::SizeFull => el = el.size_full(),
        StyleCmd::Width(v) => el = el.w(px(*v)),
        StyleCmd::Height(v) => el = el.h(px(*v)),
        StyleCmd::MinWidth(v) => el = el.min_w(px(*v)),
        StyleCmd::MinHeight(v) => el = el.min_h(px(*v)),
        StyleCmd::MaxWidth(v) => el = el.max_w(px(*v)),
        StyleCmd::MinHeightZero => el = el.min_h_0(),
        StyleCmd::MinWidthZero => el = el.min_w_0(),
        StyleCmd::TextSize(v) => el = el.text_size(px(*v)),
        StyleCmd::TextColor(color) => el = el.text_color(resolve_node_color(*color, cx)),
        StyleCmd::TextAlignRight => el = el.text_right(),
        StyleCmd::FontWeight(weight) => el = el.font_weight(font_weight(*weight)),
        StyleCmd::FontMono => el = el.font_family("monospace"),
        StyleCmd::Background(color) => el = el.bg(resolve_node_color(*color, cx)),
        StyleCmd::BorderAll => el = el.border_1(),
        StyleCmd::BorderTop => el = el.border_t_1(),
        StyleCmd::BorderBottom => el = el.border_b_1(),
        StyleCmd::BorderLeft => el = el.border_l_1(),
        StyleCmd::BorderRight => el = el.border_r_1(),
        StyleCmd::BorderColor(color) => el = el.border_color(resolve_node_color(*color, cx)),
        StyleCmd::Rounded(v) => el = el.rounded(px(*v)),
        StyleCmd::RoundedTopLeft(v) => el = el.rounded_tl(px(*v)),
        StyleCmd::RoundedFull => el = el.rounded(px(9999.0)),
        StyleCmd::Absolute => el = el.absolute(),
        StyleCmd::Relative => el = el.relative(),
        StyleCmd::Top(v) => el = el.top(px(*v)),
        StyleCmd::Right(v) => el = el.right(px(*v)),
        StyleCmd::Bottom(v) => el = el.bottom(px(*v)),
        StyleCmd::Left(v) => el = el.left(px(*v)),
        StyleCmd::CursorPointer => el = el.cursor_pointer(),
        StyleCmd::CursorDefault => el = el.cursor_default(),
        StyleCmd::OverflowHidden => el = el.overflow_hidden(),
        StyleCmd::OverflowXScroll | StyleCmd::OverflowYScroll => {
            // Wrapping commands are handled by the div renderer; here they
            // degrade to clipping so late commands on wrapped elements stay
            // total.
            el = el.overflow_hidden();
        }
        StyleCmd::ShadowSm => el = el.shadow_sm(),
        StyleCmd::ShadowMd => el = el.shadow_md(),
        StyleCmd::ShadowLg => el = el.shadow_lg(),
        StyleCmd::Opacity(v) => el = el.opacity(*v),
        StyleCmd::Hover(cmds) => {
            let cmds = cmds.clone();
            el = el.hover(move |style| {
                // Hover refinements never resolve theme colors through the
                // app; pre-resolved colors are applied via a direct pass.
                apply_hover_cmds(style, &cmds, cx)
            });
        }
    }
    el
}

fn apply_hover_cmds(
    mut style: gpui::StyleRefinement,
    cmds: &[StyleCmd],
    cx: &App,
) -> gpui::StyleRefinement {
    for cmd in cmds {
        match cmd {
            StyleCmd::Background(color) => {
                style.background = Some(resolve_node_color(*color, cx).into());
            }
            StyleCmd::TextColor(color) => {
                style.text.color = Some(resolve_node_color(*color, cx));
            }
            _ => {}
        }
    }
    style
}

// -----------------------------------------------------------------------------
// Tree rendering
// -----------------------------------------------------------------------------

/// The mid-render accumulator for a div: either the plain div or the
/// scrollable wrapper created by an `Overflow*Scroll` command (mirroring
/// the mid-chain `overflow_*_scrollbar()` calls of framework code).
enum DivAcc {
    Plain(Option<gpui::Div>),
    ScrollX(Option<Scrollable<gpui::Div>>),
    ScrollY(Option<Scrollable<gpui::Div>>),
}

impl DivAcc {
    fn new() -> Self {
        Self::Plain(Some(div()))
    }

    fn apply(&mut self, cmd: &StyleCmd, cx: &App) {
        match self {
            DivAcc::Plain(slot) => {
                let d = slot.take().expect("div present");
                *slot = Some(apply_style_cmd(d, cmd, cx));
            }
            DivAcc::ScrollX(slot) | DivAcc::ScrollY(slot) => {
                let s = slot.take().expect("scrollable present");
                *slot = Some(apply_style_cmd(s, cmd, cx));
            }
        }
    }

    fn wrap_x(&mut self) {
        if let DivAcc::Plain(slot) = self {
            let d = slot.take().expect("div present");
            *self = DivAcc::ScrollX(Some(d.overflow_x_scrollbar()));
        }
    }

    fn wrap_y(&mut self) {
        if let DivAcc::Plain(slot) = self {
            let d = slot.take().expect("div present");
            *self = DivAcc::ScrollY(Some(d.overflow_y_scrollbar()));
        }
    }

    fn push_child(&mut self, child: AnyElement) {
        match self {
            DivAcc::Plain(slot) => {
                slot.as_mut()
                    .expect("div present")
                    .extend(std::iter::once(child));
            }
            DivAcc::ScrollX(slot) | DivAcc::ScrollY(slot) => {
                slot.as_mut()
                    .expect("scrollable present")
                    .extend(std::iter::once(child));
            }
        }
    }
}

/// Renders any node. Stateful widgets (inputs, tables, virtual lists) go
/// through `state`; everything else is stateless. `view` anchors virtual
/// lists (the renderer's internal host view).
fn render_node_impl(
    node: &Node,
    view: &Entity<RendererHostView>,
    state: &mut RendererState,
    window: &mut gpui::Window,
    cx: &mut App,
) -> AnyElement {
    match node {
        Node::Div(div_node) => render_div_impl(div_node, view, state, window, cx),
        Node::Text(text) => div().child(text.text.clone()).into_any_element(),
        Node::StyledText(styled) => {
            build_styled_text(styled.text.clone(), &styled.runs, cx).into_any_element()
        }
        Node::Icon(icon) => icon_element(&icon.name).into_any_element(),
        Node::Button(button_node) => {
            let inputs = std::rc::Rc::clone(&state.inputs);
            render_button_impl(button_node, &inputs, window, cx)
        }
        Node::Input(input_node) => render_input_impl(input_node, state, window, cx),
        Node::TabBar(tab_bar_node) => render_tab_bar_impl(tab_bar_node),
        Node::Table(table_node) => render_table_impl(table_node, state, window, cx),
        Node::VirtualList(list_node) => {
            render_virtual_list_impl(list_node, view, state, cx).into_any_element()
        }
        Node::Clipboard(clipboard_node) => {
            let mut chain =
                Clipboard::new(clipboard_node.id.clone()).value(clipboard_node.value.clone());
            if let Some(tooltip) = &clipboard_node.tooltip {
                chain = chain.tooltip(tooltip.clone());
            }
            chain.into_any_element()
        }
        Node::Custom(custom_node) => {
            if custom_node.type_id == TypeId::of::<GpuiCustomElement>()
                && let Some(element) = custom_node.payload.downcast_ref::<GpuiCustomElement>()
            {
                return element.render();
            }
            warn!("unrecognized custom node payload");
            div().into_any_element()
        }
        Node::ScrollRestoration(scroll_node) => NodeScrollRestorationElement {
            handle: shared_scroll_handle(&scroll_node.handle_id),
            current_key: scroll_node.current_key.clone(),
            saved_offset: scroll_node.saved_offset.clone(),
            save_offset: scroll_node.save_offset.clone(),
            skip_once: scroll_node.skip_once.clone(),
        }
        .into_any_element(),
    }
}

#[expect(clippy::too_many_lines)]
fn render_div_impl(
    div_node: &quoin_ui::DivNode,
    view: &Entity<RendererHostView>,
    state: &mut RendererState,
    window: &mut gpui::Window,
    cx: &mut App,
) -> AnyElement {
    let mut children = Vec::with_capacity(div_node.children.len());
    for child in &div_node.children {
        children.push(render_node_impl(child, view, state, window, cx));
    }

    let mut acc = DivAcc::new();
    for cmd in &div_node.style.cmds {
        match cmd {
            StyleCmd::OverflowXScroll => acc.wrap_x(),
            StyleCmd::OverflowYScroll => acc.wrap_y(),
            _ => acc.apply(cmd, cx),
        }
    }

    for child in children {
        acc.push_child(child);
    }

    // Interactive wiring. `Div` and `Scrollable<Div>` share the
    // `InteractiveElement` surface; each arm repeats the small attach
    // sequence because the trait is not object-safe.
    let inputs_map = Rc::clone(&state.inputs);
    let on_click = div_node.on_click.clone();
    let on_mouse_down = div_node.on_mouse_down.clone();
    let on_mouse_move = div_node.on_mouse_move.clone();
    let actions = div_node.actions.clone();
    let key_context = div_node.key_context.clone();
    let track_focus = div_node.track_focus;
    let div_id = div_node.id.clone();

    let focus_handle = if track_focus {
        Some(
            state
                .focus_handles
                .entry(div_id.clone().unwrap_or_default())
                .or_insert_with(|| cx.focus_handle())
                .clone(),
        )
    } else {
        None
    };

    let wired: AnyElement = match acc {
        DivAcc::Plain(mut slot) => {
            let mut d = slot.take().expect("div present");
            if let Some(context) = &key_context {
                d = d.key_context(context.as_str());
            }
            if let Some(handle) = &focus_handle {
                d = d.track_focus(handle);
            }
            if let Some(handler) = on_click.or(on_mouse_down) {
                let inputs_map = Rc::clone(&inputs_map);
                d = d.on_mouse_down(
                    MouseButton::Left,
                    move |_event: &MouseDownEvent, _window, cx| {
                        run_handler(&handler, cx, &inputs_map);
                    },
                );
            }
            if let Some(handler) = on_mouse_move {
                let inputs_map = Rc::clone(&inputs_map);
                d = d.on_mouse_move(move |_event: &MouseMoveEvent, _window, cx| {
                    run_handler(&handler, cx, &inputs_map);
                });
            }
            for (name, handler) in actions {
                let inputs_map = Rc::clone(&inputs_map);
                d = d.on_action(move |event: &NodeAction, _window, cx| {
                    if event.name.as_ref() == name.as_str() {
                        run_handler(&handler, cx, &inputs_map);
                    }
                });
            }
            d.into_any_element()
        }
        DivAcc::ScrollX(mut slot) => {
            let mut s = slot.take().expect("scrollable present");
            if let Some(handler) = on_click.or(on_mouse_down) {
                let inputs_map = Rc::clone(&inputs_map);
                s = s.on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _window, cx| {
                    run_handler(&handler, cx, &inputs_map);
                });
            }
            if let Some(handler) = on_mouse_move {
                let inputs_map = Rc::clone(&inputs_map);
                s = s.on_mouse_move(move |_: &MouseMoveEvent, _window, cx| {
                    run_handler(&handler, cx, &inputs_map);
                });
            }
            s.into_any_element()
        }
        DivAcc::ScrollY(mut slot) => {
            let mut s = slot.take().expect("scrollable present");
            if let Some(handler) = on_click.or(on_mouse_down) {
                let inputs_map = Rc::clone(&inputs_map);
                s = s.on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _window, cx| {
                    run_handler(&handler, cx, &inputs_map);
                });
            }
            if let Some(handler) = on_mouse_move {
                let inputs_map = Rc::clone(&inputs_map);
                s = s.on_mouse_move(move |_: &MouseMoveEvent, _window, cx| {
                    run_handler(&handler, cx, &inputs_map);
                });
            }
            s.into_any_element()
        }
    };

    wired
}

fn render_button_impl(
    node: &quoin_ui::ButtonNode,
    inputs: &SharedInputMap,
    _window: &mut gpui::Window,
    cx: &mut App,
) -> AnyElement {
    let mut chain = Button::new(node.id.clone());
    chain = match node.variant {
        NodeButtonVariant::Default => chain,
        NodeButtonVariant::Primary => chain.primary(),
        NodeButtonVariant::Ghost => chain.ghost(),
        NodeButtonVariant::Destructive => chain.danger(),
    };
    chain = match node.size {
        ButtonSize::Medium => chain,
        ButtonSize::Small => chain.small(),
        ButtonSize::XSmall => chain.xsmall(),
    };
    if let Some(icon) = &node.icon
        && let Some(variant) = icon_variant(icon)
    {
        chain = chain.icon(Icon::new(variant));
    }
    if let Some(label) = &node.label {
        chain = chain.label(label.clone());
    }
    if let Some(tooltip) = &node.tooltip {
        chain = chain.tooltip(tooltip.clone());
    }
    for cmd in &node.style.cmds {
        chain = apply_style_cmd(chain, cmd, cx);
    }
    if let Some(handler) = node.on_click.clone() {
        let inputs_map = Rc::clone(inputs);
        chain = chain.on_click(move |_, _window, cx: &mut App| {
            run_handler(&handler, cx, &inputs_map);
        });
    }
    if let Some(provider) = node.dropdown_menu.clone() {
        let inputs_map = Rc::clone(inputs);
        return chain
            .dropdown_menu(move |menu: PopupMenu, _window: &mut gpui::Window, _cx| {
                let mut menu = menu;
                for item in provider() {
                    let handler = item.on_click;
                    let inputs_map = Rc::clone(&inputs_map);
                    menu = menu.item(
                        PopupMenuItem::new(item.label.clone())
                            .checked(item.checked)
                            .on_click(move |_, _window, cx: &mut App| {
                                run_handler(&handler, cx, &inputs_map);
                            }),
                    );
                }
                menu
            })
            .into_any_element();
    }
    chain.into_any_element()
}

fn render_input_impl(
    node: &quoin_ui::InputNode,
    state: &mut RendererState,
    window: &mut gpui::Window,
    cx: &mut App,
) -> AnyElement {
    if !state.inputs.borrow().contains_key(&node.id) {
        let placeholder = node.placeholder.clone();
        let initial = node.value.clone();
        let entity = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            if let Some(placeholder) = placeholder {
                input_state.set_placeholder(placeholder, window, cx);
            }
            if !initial.is_empty() {
                input_state.set_value(initial, window, cx);
            }
            input_state
        });
        // Edit-time callback bridge: the observer (bound once) invokes the
        // on_change installed by the most recent render of the owning
        // input node. Callbacks mutate the panel model silently, matching
        // the classic read-at-render semantics.
        let on_change: OnChangeSlot = Rc::new(RefCell::new(None));
        let observer_on_change = Rc::clone(&on_change);
        let observer_entity = entity.clone();
        let subscription = cx.observe(&entity, move |_, cx| {
            let value = observer_entity.read(cx).value().to_string();
            if let Some(callback) = observer_on_change.borrow().as_ref() {
                callback(value);
            }
        });
        state.inputs.borrow_mut().insert(
            node.id.clone(),
            InputEntry {
                entity,
                on_change,
                subscription,
            },
        );
    }
    let entity = state
        .inputs
        .borrow()
        .get(&node.id)
        .map(|entry| entry.entity.clone())
        .expect("input just ensured");
    // Swap in the current change callback.
    {
        let current_callback = node.on_change.clone();
        let entry_on_change = state
            .inputs
            .borrow()
            .get(&node.id)
            .map(|entry| Rc::clone(&entry.on_change))
            .expect("input just ensured");
        *entry_on_change.borrow_mut() = current_callback;
    }
    // External value sync (model resets, tour hooks): push the desired
    // value into the widget when it differs from the live text.
    {
        let desired = node.value.clone();
        let current = entity.read(cx).value().to_string();
        if current != desired {
            entity.update(cx, |input_state, cx| {
                input_state.set_value(desired, window, cx);
            });
        }
    }
    let mut chain = Input::new(&entity);
    if let Some(prefix) = &node.prefix_icon
        && let Some(variant) = icon_variant(prefix)
    {
        chain = chain.prefix(Icon::new(variant));
    }
    if node.cleanable {
        chain = chain.cleanable(true);
    }
    if node.small {
        chain = chain.small();
    }
    chain.into_any_element()
}

fn render_tab_bar_impl(node: &quoin_ui::TabBarNode) -> AnyElement {
    let on_select = node.on_select.clone();
    let mut bar = TabBar::new(node.id.clone()).selected_index(node.selected);
    bar = bar.on_click(move |selected: &usize, _window, _cx| {
        on_select(*selected);
    });
    for tab in &node.tabs {
        let mut tab_el = Tab::new().label(tab.label.clone());
        if let Some(icon) = &tab.icon
            && let Some(variant) = icon_variant(icon)
        {
            tab_el = tab_el.prefix(Icon::new(variant).ml_1());
        }
        bar = bar.child(tab_el);
    }
    bar.into_any_element()
}

fn render_table_impl(
    node: &quoin_ui::TableNode,
    state: &mut RendererState,
    window: &mut gpui::Window,
    cx: &mut App,
) -> AnyElement {
    if !state.tables.contains_key(&node.id) {
        let delegate = QuoinNodeTableDelegate {
            columns: node.columns.clone(),
            row_count: node.row_count,
            cell: node.cell.clone(),
            on_sort: node.on_sort.clone(),
            inputs: Rc::clone(&state.inputs),
        };
        let entity: Entity<TableState<QuoinNodeTableDelegate>> =
            cx.new(|cx| TableState::new(delegate, window, cx));
        state.tables.insert(node.id.clone(), entity);
    }
    let entity = state
        .tables
        .get(&node.id)
        .cloned()
        .expect("table just ensured");
    // Refresh row count + closures each render; column metadata is fixed.
    entity.update(cx, |table_state, _| {
        let delegate = table_state.delegate_mut();
        delegate.row_count = node.row_count;
        delegate.cell = node.cell.clone();
        delegate.on_sort.clone_from(&node.on_sort);
    });

    let mut table = DataTable::new(&entity)
        .stripe(node.stripe)
        .bordered(node.bordered);
    if node.small {
        table = table.with_size(ComponentSize::Small);
    }
    table.into_any_element()
}

fn render_virtual_list_impl(
    node: &quoin_ui::VirtualListNode,
    view: &Entity<RendererHostView>,
    state: &mut RendererState,
    _cx: &mut App,
) -> VirtualList {
    let handle = state
        .virtual_list_handles
        .entry(node.id.clone())
        .or_insert_with(VirtualListScrollHandle::new)
        .clone();
    let sizes = Rc::new(vec![
        Size::new(px(node.item_width), px(node.item_height));
        node.count
    ]);
    let render_item = node.render_item.clone();
    let view_for_list = view.clone();
    v_virtual_list(
        view_for_list.clone(),
        node.id.clone(),
        sizes,
        move |_view, visible_range, window, cx| {
            visible_range
                .map(|ix| {
                    let row = render_item(ix);
                    render_node_impl(&row, &view_for_list, &mut RendererState::new(), window, cx)
                })
                .collect::<Vec<_>>()
        },
    )
    .track_scroll(&handle)
}

// -----------------------------------------------------------------------------
// Styled text
// -----------------------------------------------------------------------------

/// Builds a GPUI `StyledText` from framework-agnostic runs, resolving
/// colors through the live theme.
#[must_use]
pub fn build_styled_text(
    text: impl Into<SharedString>,
    runs: &[quoin_ui::StyledRun],
    cx: &App,
) -> StyledText {
    let text = text.into();
    // Run colors resolve against the host application's live theme, exactly
    // like the classic gpui-native devtools read `cx.theme()` at render
    // time; raw hex values pass through untouched.
    let gpui_runs: Vec<gpui::TextRun> = runs
        .iter()
        .map(|run| {
            let mut style = TextStyle::default();
            if let Some(color) = run.color {
                style.color = resolve_node_color(color, cx);
            }
            if let Some(bg) = run.bg {
                style.background_color = Some(resolve_node_color(bg, cx));
            }
            style.to_run(run.len)
        })
        .collect();
    StyledText::new(text).with_runs(gpui_runs)
}

// -----------------------------------------------------------------------------
// Scroll-restoration element (port of the classic navi element)
// -----------------------------------------------------------------------------

/// The GPUI element implementing the save/restore paint cycle.
struct NodeScrollRestorationElement {
    handle: gpui::ScrollHandle,
    current_key: Rc<dyn Fn() -> Option<String>>,
    saved_offset: SavedOffsetFn,
    save_offset: SaveOffsetFn,
    skip_once: Rc<dyn Fn() -> bool>,
}

impl gpui::Element for NodeScrollRestorationElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let layout_id = window.request_layout(gpui::Style::default(), [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _state: &mut Self::RequestLayoutState,
        _window: &mut gpui::Window,
        _cx: &mut App,
    ) {
        if let Some(key) = (self.current_key)() {
            let offset = self.handle.offset().y.as_f32();
            if offset != 0.0 {
                (self.save_offset)(&key, offset);
            }
        }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _state: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        _cx: &mut App,
    ) {
        if (self.skip_once)() {
            return;
        }
        let Some(key) = (self.current_key)() else {
            return;
        };
        if let Some(saved) = (self.saved_offset)(&key) {
            let scroll_handle = self.handle.clone();
            let target = gpui::point(scroll_handle.offset().x, px(saved));
            window.on_next_frame(move |window, _| {
                scroll_handle.set_offset(target);
                window.refresh();
            });
        }
    }
}

impl gpui::IntoElement for NodeScrollRestorationElement {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

// -----------------------------------------------------------------------------
// Framework-side wrapper: render a Node inside framework-native code
// -----------------------------------------------------------------------------

/// Internal empty view used as the invalidation anchor for virtual lists
/// rendered through a [`NodeRenderer`]; its item closures never read it.
struct RendererHostView;

impl gpui::Render for RendererHostView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
    }
}

thread_local! {
    static NODE_HOST_VIEW: RefCell<Option<Entity<RendererHostView>>> = const { RefCell::new(None) };
}

/// The cached host-view entity for the current thread (created lazily
/// against the live app context).
fn node_host_view(cx: &mut App) -> Entity<RendererHostView> {
    NODE_HOST_VIEW.with(|slot| {
        if let Some(entity) = slot.borrow().as_ref() {
            return entity.clone();
        }
        let entity = cx.new(|_| RendererHostView);
        *slot.borrow_mut() = Some(entity.clone());
        entity
    })
}

/// Renders a [`quoin_ui::Node`] inside framework-native element chains.
///
/// The host application's pages stay framework-native; navi's framework-
/// agnostic components (`Link`, `Outlet`, `Awaited`, …) embed into them
/// through this wrapper.
pub struct NodeElement(pub Node);

impl From<Node> for NodeElement {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl gpui::IntoElement for NodeElement {
    type Element = NodeElementRenderer;

    fn into_element(self) -> Self::Element {
        NodeElementRenderer {
            node: self.0,
            element: None,
        }
    }
}

/// The gpui element materializing a `Node` at request-layout time.
pub struct NodeElementRenderer {
    node: Node,
    element: Option<AnyElement>,
}

impl gpui::IntoElement for NodeElementRenderer {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl gpui::Element for NodeElementRenderer {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let view = node_host_view(cx);
        let mut state = RendererState::new();
        let mut element = render_node_impl(&self.node, &view, &mut state, window, cx);
        let layout_id = element.request_layout(window, cx);
        self.element = Some(element);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _state: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        if let Some(element) = self.element.as_mut() {
            element.prepaint(window, cx);
        }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        _state: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        if let Some(element) = self.element.as_mut() {
            element.paint(window, cx);
        }
    }
}
