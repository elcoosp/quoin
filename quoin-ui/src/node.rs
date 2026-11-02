//! Framework-agnostic runtime UI tree.
//!
//! The [`Node`] model is the runtime counterpart of the `quoin_render!`
//! DSL: where the macros translate a declarative tree into framework code
//! at compile time, a `Node` tree is plain data built at runtime, so a
//! library can describe its UI without naming any GUI framework. Each
//! adapter (see `quoin-ui-gpui`) walks the tree and materializes native
//! widgets.
//!
//! The contract deliberately mirrors the subset of GPUI/gpui-component
//! machinery that complex panels need (devtools-style tabs, tables,
//! virtual lists, inputs, dropdown menus, styled text, keybindings), so a
//! panel written once against `Node` renders identically on every adapter.
//!
//! # Threading
//!
//! Handlers are `Rc` (UI-thread) closures receiving a [`NodeEventCtx`];
//! adapter state (input entities, table states, scroll handles) lives in
//! the adapter's renderer, keyed by stable node ids.

use crate::TextColor;
use std::rc::Rc;

/// A UI-thread event handler invoked by the adapter with an event context.
pub type NodeHandler = Rc<dyn Fn(&mut NodeEventCtx)>;

/// Read-back bridge for native input widgets owned by an adapter.
///
/// Native text inputs hold their live value in adapter state (e.g. a GPUI
/// `InputState` entity). A framework-agnostic panel that must observe the
/// *current* text (search filtering, route-tester submission) receives an
/// `Rc<dyn InputValues>` from the host at render time instead of touching
/// adapter types.
pub trait InputValues {
    /// The live value of the input registered under `id` (empty when the
    /// adapter has no such input).
    fn value(&self, id: &str) -> String;
}

/// A closure producing a `Node` (used for lazy fallbacks and item renderers).
pub type NodeFactory = Rc<dyn Fn() -> Node>;

/// Font weight, framework-agnostic spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NodeFontWeight {
    /// Thin (100).
    Thin,
    /// Light (300).
    Light,
    /// Normal (400).
    #[default]
    Normal,
    /// Medium (500).
    Medium,
    /// Semibold (600).
    Semibold,
    /// Bold (700).
    Bold,
}

/// One structured style command. Adapters apply commands in order, so a
/// later command can override an earlier one exactly like chained builder
/// calls in framework code.
#[derive(Clone, Debug)]
pub enum StyleCmd {
    /// `display: flex` row layout.
    Flex,
    /// Flex column direction.
    FlexCol,
    /// Flex row direction (explicit).
    FlexRow,
    /// Allow flex children to wrap.
    FlexWrap,
    /// `flex: 1 1 0%`.
    Flex1,
    /// Align items to the cross-axis center.
    ItemsCenter,
    /// Align items to the cross-axis start.
    ItemsStart,
    /// Align items to the cross-axis end.
    ItemsEnd,
    /// Justify content along the main axis: center.
    JustifyCenter,
    /// Justify content along the main axis: start.
    JustifyStart,
    /// Justify content along the main axis: end.
    JustifyEnd,
    /// Justify content along the main axis: space between.
    JustifyBetween,
    /// Gap between flex children, in logical pixels.
    Gap(f32),
    /// Padding on all sides.
    Padding(f32),
    /// Horizontal padding.
    PaddingX(f32),
    /// Vertical padding.
    PaddingY(f32),
    /// Left padding.
    PaddingLeft(f32),
    /// Right padding.
    PaddingRight(f32),
    /// Top padding.
    PaddingTop(f32),
    /// Bottom padding.
    PaddingBottom(f32),
    /// Top margin.
    MarginTop(f32),
    /// Bottom margin.
    MarginBottom(f32),
    /// Left margin.
    MarginLeft(f32),
    /// Right margin.
    MarginRight(f32),
    /// Auto left margin (push element right).
    MarginLeftAuto,
    /// Fill the main axis.
    WidthFull,
    /// Fill the cross axis.
    HeightFull,
    /// Fill both axes.
    SizeFull,
    /// Fixed width in logical pixels.
    Width(f32),
    /// Fixed height in logical pixels.
    Height(f32),
    /// Minimum width in logical pixels.
    MinWidth(f32),
    /// Minimum height in logical pixels.
    MinHeight(f32),
    /// Maximum width in logical pixels.
    MaxWidth(f32),
    /// `min-height: 0` (allow flex children to shrink below content).
    MinHeightZero,
    /// `min-width: 0`.
    MinWidthZero,
    /// Text size in logical pixels.
    TextSize(f32),
    /// Text color.
    TextColor(TextColor),
    /// Right-align text.
    TextAlignRight,
    /// Font weight.
    FontWeight(NodeFontWeight),
    /// Monospaced font family.
    FontMono,
    /// Background color.
    Background(TextColor),
    /// 1px border on all sides.
    BorderAll,
    /// 1px border on the top side only.
    BorderTop,
    /// 1px border on the bottom side only.
    BorderBottom,
    /// 1px border on the left side only.
    BorderLeft,
    /// 1px border on the right side only.
    BorderRight,
    /// Border color for the sides currently enabled.
    BorderColor(TextColor),
    /// Uniform corner radius in logical pixels.
    Rounded(f32),
    /// Top-left corner radius in logical pixels.
    RoundedTopLeft(f32),
    /// Fully rounded (pill).
    RoundedFull,
    /// Absolutely positioned within the nearest positioned ancestor.
    Absolute,
    /// Establish a positioning context for absolute children.
    Relative,
    /// Offset from the top edge (absolute positioning).
    Top(f32),
    /// Offset from the right edge (absolute positioning).
    Right(f32),
    /// Offset from the bottom edge (absolute positioning).
    Bottom(f32),
    /// Offset from the left edge (absolute positioning).
    Left(f32),
    /// Pointer cursor.
    CursorPointer,
    /// Default cursor.
    CursorDefault,
    /// Clip overflowing content.
    OverflowHidden,
    /// Horizontal scrolling with a scrollbar.
    OverflowXScroll,
    /// Vertical scrolling with a scrollbar.
    OverflowYScroll,
    /// Small drop shadow.
    ShadowSm,
    /// Medium drop shadow.
    ShadowMd,
    /// Large drop shadow.
    ShadowLg,
    /// Element opacity in `0.0..=1.0`.
    Opacity(f32),
    /// Styles applied while the pointer hovers the element.
    Hover(Vec<StyleCmd>),
}

/// Structured style for a [`Node`]: an ordered list of style commands.
#[derive(Clone, Debug, Default)]
pub struct NodeStyle {
    /// Ordered style commands.
    pub cmds: Vec<StyleCmd>,
}

impl NodeStyle {
    /// An empty style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one style command (builder style).
    #[must_use]
    pub fn with(mut self, cmd: StyleCmd) -> Self {
        self.cmds.push(cmd);
        self
    }

    /// Appends several style commands (builder style).
    #[must_use]
    pub fn with_all(mut self, cmds: impl IntoIterator<Item = StyleCmd>) -> Self {
        self.cmds.extend(cmds);
        self
    }

    /// Applies a transform to the command list (builder style), e.g. the
    /// active/inactive style hooks of a `Link`.
    #[must_use]
    pub fn edited(self, f: impl FnOnce(Vec<StyleCmd>) -> Vec<StyleCmd>) -> Self {
        Self { cmds: f(self.cmds) }
    }
}

/// A keyboard shortcut binding declared by a framework-agnostic panel:
/// `keystroke` triggers `action` while `context` is the active key context.
#[derive(Clone, Debug)]
pub struct NodeKeySpec {
    /// e.g. `"cmd-shift-d"`.
    pub keystroke: String,
    /// The action name the tree's `on_action` handlers registered.
    pub action: String,
    /// Key context name, e.g. `"Devtools"`.
    pub context: String,
}

/// Platform capabilities handed to [`NodeHandler`]s by the adapter.
///
/// One trait (rather than separate clipboard/shell objects) so an adapter
/// can implement both over a single borrow of its mutable app context.
pub trait NodeEnv {
    /// Writes text to the platform clipboard.
    fn write_clipboard(&mut self, text: &str);
    /// Opens a URL with the platform handler (browser, mail client, …).
    fn open_url(&mut self, url: &str);
    /// The live value of the native input registered under `id`, when the
    /// adapter manages inputs.
    fn input_value(&self, _id: &str) -> Option<String> {
        None
    }
}

/// Event context handed to [`NodeHandler`]s by the adapter.
pub struct NodeEventCtx<'a> {
    /// Platform capabilities, when the adapter provides them.
    pub env: Option<&'a mut dyn NodeEnv>,
}

impl NodeEventCtx<'_> {
    /// An empty context without platform capabilities.
    #[must_use]
    pub fn empty() -> Self {
        Self { env: None }
    }

    /// Writes text to the platform clipboard when available.
    pub fn write_clipboard(&mut self, text: &str) {
        if let Some(env) = self.env.as_deref_mut() {
            env.write_clipboard(text);
        }
    }

    /// Opens a URL with the platform handler when available.
    pub fn open_url(&mut self, url: &str) {
        if let Some(env) = self.env.as_deref_mut() {
            env.open_url(url);
        }
    }

    /// The live value of a native input, when the adapter manages inputs.
    #[must_use]
    pub fn input_value(&self, id: &str) -> Option<String> {
        self.env.as_deref().and_then(|env| env.input_value(id))
    }
}

/// A plain text child.
#[derive(Clone, Debug)]
pub struct TextNode {
    /// The text content.
    pub text: String,
}

/// Styled text with per-run colors (search highlighting, badges, …).
#[derive(Clone)]
pub struct StyledTextNode {
    /// The full text.
    pub text: String,
    /// Runs partitioning the text; see [`crate::StyledRun`].
    pub runs: Vec<crate::StyledRun>,
}

/// A named icon glyph.
#[derive(Clone, Debug)]
pub struct IconNode {
    /// Icon name (kebab-case, e.g. `"chevron-right"`).
    pub name: String,
}

/// One dropdown/popup menu entry.
#[derive(Clone)]
pub struct MenuItem {
    /// Visible label.
    pub label: String,
    /// Whether the checkmark is shown.
    pub checked: bool,
    /// Invoked when the item is activated.
    pub on_click: NodeHandler,
}

/// A lazy dropdown menu: the adapter builds items when the menu opens, so
/// check marks always reflect the model state at open time.
pub type MenuProvider = Rc<dyn Fn() -> Vec<MenuItem>>;
/// Sort click handler: receives the column key and ascending flag.
pub type SortHandler = Rc<dyn Fn(&str, bool)>;
/// Looks up the saved scroll offset for a key.
pub type SavedOffsetFn = Rc<dyn Fn(&str) -> Option<f32>>;
/// Persists a scroll offset under a key.
pub type SaveOffsetFn = Rc<dyn Fn(&str, f32)>;

/// Button variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NodeButtonVariant {
    /// Default (outlined) variant.
    #[default]
    Default,
    /// High-emphasis filled variant.
    Primary,
    /// Transparent/borderless variant.
    Ghost,
    /// Dangerous-action variant.
    Destructive,
}

/// Button density.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// Extra small.
    XSmall,
    /// Small.
    Small,
    /// Default density.
    #[default]
    Medium,
}

/// A button (optionally icon-only, with tooltip and dropdown menu).
#[derive(Clone)]
pub struct ButtonNode {
    /// Stable id for adapter state and tests.
    pub id: String,
    /// Text label (mutually exclusive with `children` in practice).
    pub label: Option<String>,
    /// Icon glyph name.
    pub icon: Option<String>,
    /// Visual variant.
    pub variant: NodeButtonVariant,
    /// Density.
    pub size: ButtonSize,
    /// Tooltip text.
    pub tooltip: Option<String>,
    /// Extra style commands applied on top of the widget defaults (e.g.
    /// `RoundedFull`, fixed `Width`/`Height` for FABs).
    pub style: NodeStyle,
    /// Click handler.
    pub on_click: Option<NodeHandler>,
    /// When present, the button opens a dropdown menu built lazily.
    pub dropdown_menu: Option<MenuProvider>,
}

/// A text input. The adapter keeps the native input state alive across
/// renders keyed by [`InputNode::id`]; `value` is synced into the widget
/// whenever it differs (external resets, tour hooks), and edits flow back
/// through `on_change`.
#[derive(Clone)]
pub struct InputNode {
    /// Stable id; the adapter keys its native input state by it.
    pub id: String,
    /// Placeholder shown when empty.
    pub placeholder: Option<String>,
    /// Desired value (synced into the widget when different).
    pub value: String,
    /// Leading icon glyph name.
    pub prefix_icon: Option<String>,
    /// Show the clear ("x") affordance.
    pub cleanable: bool,
    /// Compact density.
    pub small: bool,
    /// Invoked on every edit with the new text.
    pub on_change: Option<Rc<dyn Fn(String)>>,
}

/// One tab of a [`TabBarNode`].
#[derive(Clone, Debug)]
pub struct TabSpec {
    /// Visible label.
    pub label: String,
    /// Leading icon glyph name.
    pub icon: Option<String>,
}

/// A tab bar (devtools-style panel switcher).
#[derive(Clone)]
pub struct TabBarNode {
    /// Stable id.
    pub id: String,
    /// Tabs in order.
    pub tabs: Vec<TabSpec>,
    /// Selected tab index.
    pub selected: usize,
    /// Invoked with the newly selected index.
    pub on_select: Rc<dyn Fn(usize)>,
}

/// A table column.
#[derive(Clone, Debug)]
pub struct ColumnSpec {
    /// Stable column key.
    pub key: String,
    /// Header label.
    pub label: String,
    /// Fixed width in logical pixels; `None` auto-sizes.
    pub width: Option<f32>,
    /// Whether the header shows a sort toggle.
    pub sortable: bool,
    /// Whether the column is user-resizable.
    pub resizable: bool,
}

/// A data table backed by adapter table state keyed by [`TableNode::id`].
///
/// Cells are `Node`s produced lazily by [`TableNode::cell`], so the adapter
/// can rebuild rows on every render without re-creating its table state.
#[derive(Clone)]
pub struct TableNode {
    /// Stable id; the adapter keys its native table state by it.
    pub id: String,
    /// Columns in order.
    pub columns: Vec<ColumnSpec>,
    /// Number of rows.
    pub row_count: usize,
    /// Builds the cell `Node` for `(row, column)`.
    pub cell: Rc<dyn Fn(usize, usize) -> Node>,
    /// Invoked with `(column_key, ascending)` when the user sorts.
    pub on_sort: Option<SortHandler>,
    /// Whether cell borders are drawn.
    pub bordered: bool,
    /// Whether rows are striped.
    pub stripe: bool,
    /// Compact density.
    pub small: bool,
}

/// A windowed (virtualized) vertical list with fixed-size rows.
#[derive(Clone)]
pub struct VirtualListNode {
    /// Stable id; the adapter keys its scroll handle by it.
    pub id: String,
    /// Number of rows.
    pub count: usize,
    /// Row width in logical pixels.
    pub item_width: f32,
    /// Row height in logical pixels.
    pub item_height: f32,
    /// Builds the row `Node` for a visible index; re-invoked per frame for
    /// the visible range only.
    pub render_item: Rc<dyn Fn(usize) -> Node>,
}

/// An icon-only "copy to clipboard" widget.
#[derive(Clone, Debug)]
pub struct ClipboardNode {
    /// Stable id.
    pub id: String,
    /// Text copied when clicked.
    pub value: String,
    /// Tooltip text.
    pub tooltip: Option<String>,
}

/// A framework-agnostic escape hatch: adapters that recognize `type_id`
/// render the payload with a renderer registered through
/// `register_custom_renderer` (adapter-side); unrecognized adapters render
/// nothing. This lets a *host application* keep framework-native pages
/// inside an otherwise framework-agnostic tree.
#[derive(Clone)]
pub struct CustomNode {
    /// Registry key (usually `TypeId::of::<T>()` of the payload).
    pub type_id: std::any::TypeId,
    /// Opaque payload.
    pub payload: Rc<dyn std::any::Any>,
}

/// A scroll container with router scroll-restoration behavior.
///
/// The framework-agnostic *policy* (which key to save under, what offset to
/// restore) is supplied by the embedder through closures; the adapter owns
/// the *mechanics* (native scroll handle, next-frame offset writes). The
/// scrolled container itself attaches to the adapter's shared handle for
/// [`ScrollRestorationNode::handle_id`] (adapter helper), so page code and
/// this node observe the same scroll offset.
#[derive(Clone)]
pub struct ScrollRestorationNode {
    /// Shared adapter handle id (resolved by adapter helper functions).
    pub handle_id: String,
    /// The scrolled content.
    pub child: Box<Node>,
    /// Returns the save key for the current location, or `None` when
    /// restoration is disabled (no save, no restore).
    pub current_key: Rc<dyn Fn() -> Option<String>>,
    /// Returns the saved offset for a key.
    pub saved_offset: SavedOffsetFn,
    /// Saves `offset` under `key`.
    pub save_offset: SaveOffsetFn,
    /// Returns `true` when the next paint must skip restoration once
    /// (scroll reset was requested).
    pub skip_once: Rc<dyn Fn() -> bool>,
}

/// The framework-agnostic UI tree.
#[derive(Clone)]
pub enum Node {
    /// A styled container.
    Div(DivNode),
    /// Plain text.
    Text(TextNode),
    /// Text with styled runs.
    StyledText(StyledTextNode),
    /// An icon glyph.
    Icon(IconNode),
    /// A button.
    Button(ButtonNode),
    /// A text input.
    Input(InputNode),
    /// A tab bar.
    TabBar(TabBarNode),
    /// A data table.
    Table(TableNode),
    /// A windowed list.
    VirtualList(VirtualListNode),
    /// A copy-to-clipboard widget.
    Clipboard(ClipboardNode),
    /// An adapter-recognized custom payload.
    Custom(CustomNode),
    /// A scroll container with restoration policy.
    ScrollRestoration(ScrollRestorationNode),
}

/// A styled container node.
#[derive(Clone, Default)]
pub struct DivNode {
    /// Stable id (used by adapters for focus/scroll bookkeeping).
    pub id: Option<String>,
    /// Style commands, applied in order.
    pub style: NodeStyle,
    /// Children in order.
    pub children: Vec<Node>,
    /// Left-button press handler (maps to `on_mouse_down(Left)` in GPUI).
    pub on_click: Option<NodeHandler>,
    /// Left-button press handler; distinct from [`DivNode::on_click`] only
    /// for documentation clarity — both map to the same event.
    pub on_mouse_down: Option<NodeHandler>,
    /// Pointer-move handler (used for intent preloading).
    pub on_mouse_move: Option<NodeHandler>,
    /// Keyboard context name for `actions`.
    pub key_context: Option<String>,
    /// Named actions handled within `key_context`.
    pub actions: Vec<(String, NodeHandler)>,
    /// Whether the adapter should track keyboard focus on this element.
    pub track_focus: bool,
}

impl Node {
    /// Builds a div node.
    #[must_use]
    pub fn div() -> DivBuilder {
        DivBuilder::default()
    }

    /// Builds a text node.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextNode { text: text.into() })
    }

    /// Builds a styled-text node.
    pub fn styled_text(text: impl Into<String>, runs: Vec<crate::StyledRun>) -> Self {
        Self::StyledText(StyledTextNode {
            text: text.into(),
            runs,
        })
    }

    /// Builds an icon node.
    pub fn icon(name: impl Into<String>) -> Self {
        Self::Icon(IconNode { name: name.into() })
    }
}

/// Builder for [`Node::Div`].
#[derive(Default)]
pub struct DivBuilder {
    node: DivNode,
}

impl DivBuilder {
    /// Sets the stable element id.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.node.id = Some(id.into());
        self
    }

    /// Appends one style command.
    #[must_use]
    pub fn style(mut self, cmd: StyleCmd) -> Self {
        self.node.style.cmds.push(cmd);
        self
    }

    /// Appends several style commands.
    #[must_use]
    pub fn styles(mut self, cmds: impl IntoIterator<Item = StyleCmd>) -> Self {
        self.node.style.cmds.extend(cmds);
        self
    }

    /// Replaces the whole style with a pre-built [`NodeStyle`].
    #[must_use]
    pub fn style_all(mut self, style: NodeStyle) -> Self {
        self.node.style = style;
        self
    }

    /// Appends a child node.
    #[must_use]
    pub fn child(mut self, child: impl Into<NodeChild>) -> Self {
        self.node.children.push(child.into().0);
        self
    }

    /// Appends many child nodes.
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.node.children.extend(children);
        self
    }

    /// Sets the left-click handler.
    #[must_use]
    pub fn on_click(mut self, f: impl Fn(&mut NodeEventCtx) + 'static) -> Self {
        self.node.on_click = Some(Rc::new(f));
        self
    }

    /// Sets the pointer-move handler.
    #[must_use]
    pub fn on_mouse_move(mut self, f: impl Fn(&mut NodeEventCtx) + 'static) -> Self {
        self.node.on_mouse_move = Some(Rc::new(f));
        self
    }

    /// Sets the keyboard context name.
    #[must_use]
    pub fn key_context(mut self, context: impl Into<String>) -> Self {
        self.node.key_context = Some(context.into());
        self
    }

    /// Registers a named action handler.
    #[must_use]
    pub fn on_action(
        mut self,
        name: impl Into<String>,
        f: impl Fn(&mut NodeEventCtx) + 'static,
    ) -> Self {
        self.node.actions.push((name.into(), Rc::new(f)));
        self
    }

    /// Tracks keyboard focus on this element.
    #[must_use]
    pub fn track_focus(mut self) -> Self {
        self.node.track_focus = true;
        self
    }

    /// Finishes the builder into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::Div(self.node)
    }
}

/// Convenience conversion so `Node` can be passed as a child directly.
pub struct NodeChild(pub Node);

impl From<Node> for NodeChild {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl From<&str> for NodeChild {
    fn from(text: &str) -> Self {
        Self(Node::text(text))
    }
}

impl From<String> for NodeChild {
    fn from(text: String) -> Self {
        Self(Node::text(text))
    }
}

impl From<&String> for NodeChild {
    fn from(text: &String) -> Self {
        Self(Node::text(text.clone()))
    }
}

impl From<DivBuilder> for NodeChild {
    fn from(builder: DivBuilder) -> Self {
        Self(builder.build())
    }
}

impl From<ButtonNode> for NodeChild {
    fn from(node: ButtonNode) -> Self {
        Self(node.build())
    }
}

impl From<InputNode> for NodeChild {
    fn from(node: InputNode) -> Self {
        Self(node.build())
    }
}

impl From<TabBarNode> for NodeChild {
    fn from(node: TabBarNode) -> Self {
        Self(node.build())
    }
}

impl From<TableNode> for NodeChild {
    fn from(node: TableNode) -> Self {
        Self(node.build())
    }
}

impl From<VirtualListNode> for NodeChild {
    fn from(node: VirtualListNode) -> Self {
        Self(node.build())
    }
}

impl From<ClipboardNode> for NodeChild {
    fn from(node: ClipboardNode) -> Self {
        Self(node.build())
    }
}

/// Builds a button node.
#[must_use]
pub fn button(id: impl Into<String>) -> ButtonNode {
    ButtonNode {
        id: id.into(),
        label: None,
        icon: None,
        variant: NodeButtonVariant::Default,
        size: ButtonSize::Medium,
        tooltip: None,
        style: NodeStyle::new(),
        on_click: None,
        dropdown_menu: None,
    }
}

impl ButtonNode {
    /// Sets the text label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the icon glyph.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Selects the primary variant.
    #[must_use]
    pub fn primary(mut self) -> Self {
        self.variant = NodeButtonVariant::Primary;
        self
    }

    /// Selects the ghost variant.
    #[must_use]
    pub fn ghost(mut self) -> Self {
        self.variant = NodeButtonVariant::Ghost;
        self
    }

    /// Selects the destructive variant.
    #[must_use]
    pub fn destructive(mut self) -> Self {
        self.variant = NodeButtonVariant::Destructive;
        self
    }

    /// Selects extra-small density.
    #[must_use]
    pub fn xsmall(mut self) -> Self {
        self.size = ButtonSize::XSmall;
        self
    }

    /// Selects small density.
    #[must_use]
    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    /// Sets the tooltip.
    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Appends extra style commands (FAB shape, shadows, …).
    #[must_use]
    pub fn styles(mut self, cmds: impl IntoIterator<Item = StyleCmd>) -> Self {
        self.style.cmds.extend(cmds);
        self
    }

    /// Sets the click handler.
    #[must_use]
    pub fn on_click(mut self, f: impl Fn(&mut NodeEventCtx) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// Attaches a lazily-built dropdown menu.
    #[must_use]
    pub fn dropdown_menu(mut self, provider: impl Fn() -> Vec<MenuItem> + 'static) -> Self {
        self.dropdown_menu = Some(Rc::new(provider));
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::Button(self)
    }
}

/// Builds an input node.
#[must_use]
pub fn input(id: impl Into<String>) -> InputNode {
    InputNode {
        id: id.into(),
        placeholder: None,
        value: String::new(),
        prefix_icon: None,
        cleanable: false,
        small: false,
        on_change: None,
    }
}

impl InputNode {
    /// Sets the placeholder.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the desired value.
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the leading icon.
    #[must_use]
    pub fn prefix_icon(mut self, icon: impl Into<String>) -> Self {
        self.prefix_icon = Some(icon.into());
        self
    }

    /// Enables the clear affordance.
    #[must_use]
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Selects compact density.
    #[must_use]
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// Sets the change handler.
    #[must_use]
    pub fn on_change(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::Input(self)
    }
}

/// Builds a tab bar node.
#[must_use]
pub fn tab_bar(id: impl Into<String>) -> TabBarNode {
    TabBarNode {
        id: id.into(),
        tabs: Vec::new(),
        selected: 0,
        on_select: Rc::new(|_| {}),
    }
}

impl TabBarNode {
    /// Appends a tab.
    #[must_use]
    pub fn tab(mut self, label: impl Into<String>, icon: Option<String>) -> Self {
        self.tabs.push(TabSpec {
            label: label.into(),
            icon,
        });
        self
    }

    /// Sets the selected index.
    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Sets the selection handler.
    #[must_use]
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Rc::new(f);
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::TabBar(self)
    }
}

/// Builds a data-table node.
#[must_use]
pub fn data_table(id: impl Into<String>) -> TableNode {
    TableNode {
        id: id.into(),
        columns: Vec::new(),
        row_count: 0,
        cell: Rc::new(|_, _| Node::div().build()),
        on_sort: None,
        bordered: false,
        stripe: false,
        small: false,
    }
}

impl TableNode {
    /// Appends a column.
    #[must_use]
    pub fn column(
        mut self,
        key: impl Into<String>,
        label: impl Into<String>,
        width: Option<f32>,
        sortable: bool,
        resizable: bool,
    ) -> Self {
        self.columns.push(ColumnSpec {
            key: key.into(),
            label: label.into(),
            width,
            sortable,
            resizable,
        });
        self
    }

    /// Sets the row count.
    #[must_use]
    pub fn row_count(mut self, count: usize) -> Self {
        self.row_count = count;
        self
    }

    /// Sets the cell renderer.
    #[must_use]
    pub fn cell(mut self, f: impl Fn(usize, usize) -> Node + 'static) -> Self {
        self.cell = Rc::new(f);
        self
    }

    /// Sets the sort handler.
    #[must_use]
    pub fn on_sort(mut self, f: impl Fn(&str, bool) + 'static) -> Self {
        self.on_sort = Some(Rc::new(f));
        self
    }

    /// Enables cell borders.
    #[must_use]
    pub fn bordered(mut self) -> Self {
        self.bordered = true;
        self
    }

    /// Enables row striping.
    #[must_use]
    pub fn stripe(mut self) -> Self {
        self.stripe = true;
        self
    }

    /// Selects compact density.
    #[must_use]
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::Table(self)
    }
}

/// Builds a virtual-list node.
#[must_use]
pub fn virtual_list(id: impl Into<String>) -> VirtualListNode {
    VirtualListNode {
        id: id.into(),
        count: 0,
        item_width: 2500.0,
        item_height: 32.0,
        render_item: Rc::new(|_| Node::div().build()),
    }
}

impl VirtualListNode {
    /// Sets the row count.
    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Overrides the row width (default 2500px, matching wide timeline rows).
    #[must_use]
    pub fn item_width(mut self, width: f32) -> Self {
        self.item_width = width;
        self
    }

    /// Overrides the row height (default 32px).
    #[must_use]
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// Sets the row renderer.
    #[must_use]
    pub fn render_item(mut self, f: impl Fn(usize) -> Node + 'static) -> Self {
        self.render_item = Rc::new(f);
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::VirtualList(self)
    }
}

/// Builds a clipboard node.
#[must_use]
pub fn clipboard(id: impl Into<String>) -> ClipboardNode {
    ClipboardNode {
        id: id.into(),
        value: String::new(),
        tooltip: None,
    }
}

impl ClipboardNode {
    /// Sets the copied text.
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the tooltip.
    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::Clipboard(self)
    }
}

/// Builds a scroll-restoration container bound to the adapter's shared
/// scroll handle `handle_id`.
#[must_use]
pub fn scroll_restoration(handle_id: impl Into<String>, child: Node) -> ScrollRestorationNode {
    ScrollRestorationNode {
        handle_id: handle_id.into(),
        child: Box::new(child),
        current_key: Rc::new(|| None),
        saved_offset: Rc::new(|_| None),
        save_offset: Rc::new(|_, _| {}),
        skip_once: Rc::new(|| false),
    }
}

impl ScrollRestorationNode {
    /// Sets the save-key provider.
    #[must_use]
    pub fn current_key(mut self, f: impl Fn() -> Option<String> + 'static) -> Self {
        self.current_key = Rc::new(f);
        self
    }

    /// Sets the saved-offset lookup.
    #[must_use]
    pub fn saved_offset(mut self, f: impl Fn(&str) -> Option<f32> + 'static) -> Self {
        self.saved_offset = Rc::new(f);
        self
    }

    /// Sets the offset saver.
    #[must_use]
    pub fn save_offset(mut self, f: impl Fn(&str, f32) + 'static) -> Self {
        self.save_offset = Rc::new(f);
        self
    }

    /// Sets the one-shot skip predicate.
    #[must_use]
    pub fn skip_once(mut self, f: impl Fn() -> bool + 'static) -> Self {
        self.skip_once = Rc::new(f);
        self
    }

    /// Finishes into a [`Node`].
    #[must_use]
    pub fn build(self) -> Node {
        Node::ScrollRestoration(self)
    }
}

/// Builds a custom node from a typed payload.
pub fn custom<T: 'static>(payload: T) -> Node {
    Node::Custom(CustomNode {
        type_id: std::any::TypeId::of::<T>(),
        payload: Rc::new(payload),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_builder_collects_children_and_styles() {
        let node = Node::div()
            .id("root")
            .style(StyleCmd::Flex)
            .style(StyleCmd::Gap(4.0))
            .child("hello")
            .child(Node::icon("info"))
            .build();

        let Node::Div(div) = node else {
            panic!("expected div");
        };
        assert_eq!(div.id.as_deref(), Some("root"));
        assert_eq!(div.style.cmds.len(), 2);
        assert_eq!(div.children.len(), 2);
        assert!(matches!(div.children[0], Node::Text(_)));
        assert!(matches!(div.children[1], Node::Icon(_)));
    }

    #[test]
    fn text_children_from_strings() {
        let owned = String::from("owned");
        let node = Node::div()
            .child("borrowed")
            .child(owned.clone())
            .child(&owned)
            .build();
        let Node::Div(div) = node else {
            panic!("expected div");
        };
        assert_eq!(div.children.len(), 3);
    }

    #[test]
    fn button_fluent_api() {
        let node = button("b1")
            .label("Save")
            .ghost()
            .small()
            .tooltip("Save changes")
            .on_click(|_| {})
            .build();
        let Node::Button(b) = node else {
            panic!("expected button");
        };
        assert_eq!(b.id, "b1");
        assert_eq!(b.label.as_deref(), Some("Save"));
        assert_eq!(b.variant, NodeButtonVariant::Ghost);
        assert_eq!(b.size, ButtonSize::Small);
        assert!(b.on_click.is_some());
    }

    #[test]
    fn custom_nodes_keep_types() {
        let node = custom(42_u32);
        let Node::Custom(c) = node else {
            panic!("expected custom");
        };
        assert_eq!(c.type_id, std::any::TypeId::of::<u32>());
        assert_eq!(c.payload.downcast_ref::<u32>(), Some(&42));
    }
}
