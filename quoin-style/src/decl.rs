//! The typed style vocabulary: one [`Decl`] per style decision.
//!
//! Every supported Tailwind class parses into exactly one `Decl` value. The
//! enum is deliberately flat — lowerings pattern-match on it and emit one
//! builder call per variant, so "which classes does backend X support" is
//! answered by reading one `match` arm instead of reverse-engineering a
//! class string at runtime.

/// A length value: logical pixels, `100%`, or `auto`.
///
/// Tailwind spacing classes (`gap-2`, `p-4`, `w-[600px]`, …) all lower into
/// [`Len::Px`]; `w-full` into [`Len::Full`]; `w-auto`/`mx-auto` into
/// [`Len::Auto`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    /// A fixed length in logical pixels.
    Px(f32),
    /// `100%` — fill the parent axis.
    Full,
    /// The `auto` keyword (content-determined, or auto margins).
    Auto,
}

impl Len {
    /// The pixel value of a spacing-scale class: `1` → 4.0 px, `2` → 8.0 px,
    /// and so on (Tailwind's `1 unit = 4 px` rule).
    #[must_use]
    pub fn spacing(units: f32) -> Self {
        Self::Px(units * 4.0)
    }
}

/// A literal sRGB color stored exactly as it appeared in source.
///
/// The text form (`"0x3b82f6"`) exists so the GPUI lowering can emit the
/// byte-identical hex literal the hand-written transpiler produced before
/// the style IR existed; the numeric form is what the Floem/Xilem lowerings
/// pack into their color types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexLit {
    /// The literal as written, without the `0x` prefix (e.g. `"3b82f6"`).
    /// Empty for arbitrary (bracket-syntax) colors, whose text form is not
    /// static.
    pub text: &'static str,
    /// The numeric value, packed `0xRRGGBBAA` (alpha in the low byte, the
    /// same layout `gpui::rgba` expects). Opaque colors have `0xff` there.
    pub value: u32,
}

impl HexLit {
    /// Builds a literal from a static hex string (no `0x` prefix).
    #[must_use]
    pub const fn new(text: &'static str, value: u32) -> Self {
        Self { text, value }
    }

    /// The red channel (0-255).
    #[must_use]
    pub const fn red(self) -> u8 {
        ((self.value >> 24) & 0xff) as u8
    }

    /// The green channel (0-255).
    #[must_use]
    pub const fn green(self) -> u8 {
        ((self.value >> 16) & 0xff) as u8
    }

    /// The blue channel (0-255).
    #[must_use]
    pub const fn blue(self) -> u8 {
        ((self.value >> 8) & 0xff) as u8
    }

    /// The alpha channel (0-255); opaque colors report 255.
    #[must_use]
    pub const fn alpha(self) -> u8 {
        (self.value & 0xff) as u8
    }
}

/// A color specification: a literal RGB value, `transparent`, or a theme
/// token resolved by the backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Col {
    /// A literal sRGB color (`bg-gray-500`, `bg-[#1e2e3e]`).
    Rgb(HexLit),
    /// Fully transparent (`bg-transparent`).
    Transparent,
    /// GPUI's `white()` — kept as its own variant so the GPUI lowering
    /// stays byte-identical to the pre-IR transpiler.
    White,
    /// GPUI's `black()`.
    Black,
    /// A theme token (`bg-primary`): resolved per backend.
    Token(Token),
}

/// A theme token name. Fifteen tokens, matching `quoin_ui::ThemeToken` and
/// the per-adapter `theme` modules (one atomic per token, no locks).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token {
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
    /// Card surface color.
    Card,
}

impl Token {
    /// Every token, in declaration order. Adapter `theme` modules expose one
    /// function per entry of this list.
    pub const ALL: [Self; 15] = [
        Self::Primary,
        Self::Secondary,
        Self::Background,
        Self::Foreground,
        Self::Muted,
        Self::MutedForeground,
        Self::Accent,
        Self::Info,
        Self::Success,
        Self::Warning,
        Self::Danger,
        Self::Border,
        Self::Input,
        Self::Ring,
        Self::Card,
    ];

    /// The `snake_case` function name an adapter's `theme` module exposes for
    /// this token (e.g. [`Token::MutedForeground`] → `"muted_foreground"`).
    #[must_use]
    pub const fn fn_name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Background => "background",
            Self::Foreground => "foreground",
            Self::Muted => "muted",
            Self::MutedForeground => "muted_foreground",
            Self::Accent => "accent",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Border => "border",
            Self::Input => "input",
            Self::Ring => "ring",
            Self::Card => "card",
        }
    }

    /// The default surface (background/border role) value of this token in
    /// the shadcn dark palette that the GPUI backend baked in before the
    /// style IR existed.
    ///
    /// These values are also the initial contents of the Floem and Xilem
    /// adapter theme atomics, so a fresh application looks the same on all
    /// backends until `set_palette` is called.
    #[must_use]
    pub const fn surface_hex(self) -> HexLit {
        match self {
            Self::Primary | Self::Accent | Self::Ring => HexLit::new("6366f1", 0x63_66_f1_ff),
            Self::Secondary
            | Self::Muted
            | Self::MutedForeground
            | Self::Border
            | Self::Card
            | Self::Input => HexLit::new("1e293b", 0x1e_29_3b_ff),
            Self::Background => HexLit::new("0f172a", 0x0f_17_2a_ff),
            Self::Foreground => HexLit::new("fafafa", 0xfa_fa_fa_ff),
            Self::Info => HexLit::new("1e3a5f", 0x1e_3a_5f_ff),
            Self::Success => HexLit::new("14532d", 0x14_53_2d_ff),
            Self::Warning => HexLit::new("422006", 0x42_20_06_ff),
            Self::Danger => HexLit::new("450a0a", 0x45_0a_0a_ff),
        }
    }

    /// The default text-role value of this token in the same palette.
    ///
    /// The pre-IR GPUI transpiler approximated `text-secondary` and friends
    /// with lighter palette colors than their background counterparts (a
    /// de-emphasized label is not the surface color); those approximations
    /// are preserved here for byte-identical GPUI output.
    #[must_use]
    pub const fn text_hex(self) -> HexLit {
        match self {
            Self::Primary | Self::Accent => HexLit::new("6366f1", 0x63_66_f1_ff),
            Self::Secondary | Self::Muted | Self::MutedForeground => {
                HexLit::new("94a3b8", 0x94_a3_b8_ff)
            }
            Self::Background | Self::Foreground => HexLit::new("fafafa", 0xfa_fa_fa_ff),
            Self::Info => HexLit::new("3b82f6", 0x3b_82_f6_ff),
            Self::Success => HexLit::new("22c55e", 0x22_c5_5e_ff),
            Self::Warning => HexLit::new("eab308", 0xea_b3_08_ff),
            Self::Danger => HexLit::new("ef4444", 0xef_44_44_ff),
            Self::Border => HexLit::new("e2e8f0", 0xe2_e8_f0_ff),
            Self::Input | Self::Ring | Self::Card => HexLit::new("1e293b", 0x1e_29_3b_ff),
        }
    }
}

/// Flex container direction (`flex-row` / `flex-col` and their reverses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDir {
    /// Main axis left-to-right.
    Row,
    /// Main axis top-to-bottom.
    Column,
    /// Main axis right-to-left.
    RowReverse,
    /// Main axis bottom-to-top.
    ColumnReverse,
}

/// CSS `display` value. Backends that have no concept of non-flex display
/// (Floem and Xilem are always flex) ignore every variant except
/// [`Display::Hidden`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    /// Flex container (row direction is the GPUI default).
    Flex,
    /// Inline flex container.
    InlineFlex,
    /// Block.
    Block,
    /// Inline block.
    InlineBlock,
    /// Hidden (`display: none`).
    Hidden,
}

/// Cross-axis alignment of flex children (`items-*`, `self-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Items {
    /// Pack to the start.
    Start,
    /// Pack to the end.
    End,
    /// Center.
    Center,
    /// Baseline alignment.
    Baseline,
    /// Stretch to fill.
    Stretch,
}

/// Main-axis distribution of flex children (`justify-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    /// Pack to the start.
    Start,
    /// Pack to the end.
    End,
    /// Center.
    Center,
    /// Space between.
    Between,
    /// Space around.
    Around,
    /// Space evenly.
    Evenly,
}

/// Which border side a width or color applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// Which border side a width applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderSide {
    /// All four sides.
    All,
    /// Bottom only.
    Bottom,
    /// Top only.
    Top,
    /// Left only.
    Left,
    /// Right only.
    Right,
}

/// Box-shadow strength (`shadow-xs` … `shadow-xl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shadow {
    /// `shadow-xs`.
    Xs,
    /// `shadow-sm`.
    Sm,
    /// `shadow-md`.
    Md,
    /// `shadow-lg`.
    Lg,
    /// `shadow-xl`.
    Xl,
}

/// Mouse cursor style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    /// Pointer (hand).
    Pointer,
    /// Default arrow.
    Default,
    /// Text (I-beam).
    Text,
}

/// Scroll overflow mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Clip children.
    Hidden,
    /// Scroll on the y axis.
    ScrollY,
    /// Scroll on the x axis.
    ScrollX,
    /// Scroll both axes.
    Scroll,
    /// Auto-scroll on the y axis (floem treats this as a scroll container).
    AutoY,
    /// Auto-scroll on the x axis.
    AutoX,
}

impl Overflow {
    /// Whether the GPUI lowering needs a stateful element: scroll methods
    /// live on `StatefulInteractiveElement`, so the target div must carry an
    /// element id before they can be applied.
    #[must_use]
    pub fn needs_stateful_scroll(self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

/// Positioning scheme (`relative`, `absolute`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    /// In-flow.
    Relative,
    /// Out-of-flow, anchored to the nearest positioned ancestor.
    Absolute,
}

/// Text alignment inside the element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    /// Left-aligned.
    Left,
    /// Centered.
    Center,
    /// Right-aligned.
    Right,
}

/// The Tailwind font-size scale. Parsed into px values for the
/// framework-agnostic lowerings; the GPUI lowering maps the canonical px
/// values back onto its named methods to preserve byte-identical output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextScale {
    /// `text-xs` → 12 px.
    Xs,
    /// `text-sm` → 14 px.
    Sm,
    /// `text-base` → 16 px.
    Base,
    /// `text-lg` → 18 px.
    Lg,
    /// `text-xl` → 20 px.
    Xl,
    /// `text-2xl` → 24 px.
    Xxl,
    /// `text-3xl` → 30 px.
    Xxl3,
    /// `text-4xl` → 36 px.
    Xxl4,
}

impl TextScale {
    /// The pixel size of this scale step.
    #[must_use]
    pub const fn px(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm => 14.0,
            Self::Base => 16.0,
            Self::Lg => 18.0,
            Self::Xl => 20.0,
            Self::Xxl => 24.0,
            Self::Xxl3 => 30.0,
            Self::Xxl4 => 36.0,
        }
    }

    /// The scale step for a canonical px value, if it is one of the named
    /// steps (used by the GPUI lowering to keep emitting `text_xs()`-style
    /// methods for scale classes).
    #[must_use]
    pub const fn from_px(px: f32) -> Option<Self> {
        match px {
            12.0 => Some(Self::Xs),
            14.0 => Some(Self::Sm),
            16.0 => Some(Self::Base),
            18.0 => Some(Self::Lg),
            20.0 => Some(Self::Xl),
            24.0 => Some(Self::Xxl),
            30.0 => Some(Self::Xxl3),
            36.0 => Some(Self::Xxl4),
            _ => None,
        }
    }
}

/// Font weight as a numeric value (Tailwind's 100–900 scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weight(pub u16);

impl Weight {
    /// `font-thin` → 100.
    pub const THIN: Self = Self(100);
    /// `font-light` → 300.
    pub const LIGHT: Self = Self(300);
    /// `font-normal` → 400.
    pub const NORMAL: Self = Self(400);
    /// `font-medium` → 500.
    pub const MEDIUM: Self = Self(500);
    /// `font-semibold` → 600.
    pub const SEMIBOLD: Self = Self(600);
    /// `font-bold` → 700.
    pub const BOLD: Self = Self(700);

    /// The weight class for a named Tailwind weight, if `value` is one.
    #[must_use]
    pub const fn from_name(value: u16) -> Option<Self> {
        match value {
            100 => Some(Self::THIN),
            300 => Some(Self::LIGHT),
            400 => Some(Self::NORMAL),
            500 => Some(Self::MEDIUM),
            600 => Some(Self::SEMIBOLD),
            700 => Some(Self::BOLD),
            _ => None,
        }
    }
}

/// One typed style decision: what a single Tailwind class means.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Decl {
    // ── Layout ──────────────────────────────────────────────────────────
    /// CSS `display` value (`flex`, `hidden`, …).
    Display(Display),
    /// Flex container direction (`flex-col`, `flex-row`, …).
    FlexDir(FlexDir),
    /// Allow children to wrap (`flex-wrap`); `false` for `flex-nowrap`.
    FlexWrap(bool),
    /// Main-axis distribution (`justify-center`, …).
    Justify(Justify),
    /// Cross-axis alignment (`items-center`, …).
    Items(Items),
    /// Self alignment (`self-center`, …).
    SelfAlign(Items),
    /// Flex grow factor (`grow` → 1.0, `grow-0` → 0.0).
    Grow(f32),
    /// Flex shrink factor (`shrink` → 1.0, `shrink-0` → 0.0).
    Shrink(f32),
    /// `flex: 1 1 0%` (`flex-1`).
    Flex1,
    /// Gap between flex children (`gap-2`, `gap-[6px]`).
    Gap(Len),

    // ── Sizing ──────────────────────────────────────────────────────────
    /// Width (`w-full`, `w-auto`, `w-[600px]`).
    Width(Len),
    /// Height (`h-full`, `h-auto`, `h-[40px]`).
    Height(Len),
    /// Fill both axes (`size-full`).
    SizeFull,
    /// Minimum width (`min-w-0`, `min-w-[100px]`).
    MinWidth(Len),
    /// Minimum height (`min-h-0`, `min-h-[100px]`).
    MinHeight(Len),
    /// Maximum width (`max-w-full`, `max-w-[600px]`).
    MaxWidth(Len),
    /// Maximum height (`max-h-full`, `max-h-[600px]`).
    MaxHeight(Len),

    // ── Spacing ─────────────────────────────────────────────────────────
    /// Padding, all sides (`p-4`).
    PaddingAll(Len),
    /// Horizontal padding (`px-2`).
    PaddingX(Len),
    /// Vertical padding (`py-2`).
    PaddingY(Len),
    /// Top padding (`pt-2`).
    PaddingTop(Len),
    /// Bottom padding (`pb-2`).
    PaddingBottom(Len),
    /// Left padding (`pl-2`).
    PaddingLeft(Len),
    /// Right padding (`pr-2`).
    PaddingRight(Len),
    /// Margin, all sides (`m-4`).
    MarginAll(Len),
    /// Horizontal margin (`mx-4`); [`Len::Auto`] for `mx-auto`.
    MarginX(Len),
    /// Vertical margin (`my-4`).
    MarginY(Len),
    /// Top margin (`mt-2`).
    MarginTop(Len),
    /// Bottom margin (`mb-2`).
    MarginBottom(Len),
    /// Left margin (`ml-2`); [`Len::Auto`] for `ml-auto`.
    MarginLeft(Len),
    /// Right margin (`mr-2`); [`Len::Auto`] for `mr-auto`.
    MarginRight(Len),

    // ── Position & overflow ─────────────────────────────────────────────
    /// Positioning scheme (`relative`, `absolute`).
    Position(Position),
    /// `top-…` offset.
    Top(Len),
    /// `right-…` offset.
    Right(Len),
    /// `bottom-…` offset.
    Bottom(Len),
    /// `left-…` offset.
    Left(Len),
    /// Overflow behavior (`overflow-hidden`, `overflow-y-scroll`, …).
    Overflow(Overflow),

    // ── Visual ──────────────────────────────────────────────────────────
    /// Background color (`bg-primary`, `bg-[#112233]`, …).
    Background(Col),
    /// Border color (`border-gray-300`, …).
    BorderColor(Col),
    /// Border width on one side (`border`, `border-b-2`, …).
    BorderWidth(BorderSide, f32),
    /// Corner radius, all corners (`rounded-lg`, `rounded-[10px]`).
    Rounded(f32),
    /// Corner radius, one corner (`rounded-tl-lg`, …).
    RoundedCorner(Corner, f32),
    /// Box shadow (`shadow-md`).
    Shadow(Shadow),
    /// Opacity 0.0–1.0 (`opacity-50`, `opacity-[0.9]`).
    Opacity(f32),
    /// Mouse cursor (`cursor-pointer`).
    Cursor(Cursor),

    // ── Text (inherited by text leaves) ─────────────────────────────────
    /// Text color (`text-primary`, `text-[#aabbcc]`, …).
    TextColor(Col),
    /// Font size (`text-lg`, `text-[11px]`).
    TextSize(f32),
    /// Font weight (`font-semibold`, …).
    TextWeight(Weight),
    /// Text alignment (`text-center`).
    TextAlign(TextAlign),
    /// `whitespace-nowrap`.
    WhitespaceNowrap,
    /// `truncate` (nowrap + overflow hidden).
    Truncate,
    /// Font family (`font-mono` → `"monospace"`).
    FontFamily(&'static str),
    /// `select-none`.
    SelectNone,
}

impl Decl {
    /// Whether this property must cascade down to every text leaf for the
    /// Xilem lowering, which has no style inheritance: the backend threads
    /// the cascaded text state through its compile-time context instead of
    /// emitting per-container properties.
    ///
    /// Alignment is intentionally absent: Masonry models alignment per
    /// text widget, but the value is chosen by the leaf's own lowering from
    /// the container context — it is cascade-*aware* without being
    /// cascade-*required* on containers that contain no text.
    #[must_use]
    pub fn is_inherited_text(&self) -> bool {
        matches!(
            self,
            Self::TextColor(_) | Self::TextSize(_) | Self::TextWeight(_)
        )
    }

    /// The human-readable meaning of this decl, used by the semantic probe
    /// (section 7 of the rendering-parity plan) so golden files describe
    /// styles framework-independently.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Display(d) => format!("display:{d:?}"),
            Self::FlexDir(d) => format!("flex-dir:{d:?}"),
            Self::FlexWrap(w) => format!("flex-wrap:{w}"),
            Self::Justify(j) => format!("justify:{j:?}"),
            Self::Items(i) => format!("items:{i:?}"),
            Self::SelfAlign(i) => format!("self:{i:?}"),
            Self::Grow(v) => format!("grow:{v}"),
            Self::Shrink(v) => format!("shrink:{v}"),
            Self::Flex1 => "flex-1".to_owned(),
            Self::Gap(Len::Px(v)) => format!("gap:{v}"),
            Self::Gap(l) => format!("gap:{l:?}"),
            Self::Width(l) => format!("width:{l:?}"),
            Self::Height(l) => format!("height:{l:?}"),
            Self::SizeFull => "size-full".to_owned(),
            Self::MinWidth(l) => format!("min-width:{l:?}"),
            Self::MinHeight(l) => format!("min-height:{l:?}"),
            Self::MaxWidth(l) => format!("max-width:{l:?}"),
            Self::MaxHeight(l) => format!("max-height:{l:?}"),
            Self::PaddingAll(l) => format!("padding:{l:?}"),
            Self::PaddingX(l) => format!("padding-x:{l:?}"),
            Self::PaddingY(l) => format!("padding-y:{l:?}"),
            Self::PaddingTop(l) => format!("padding-top:{l:?}"),
            Self::PaddingBottom(l) => format!("padding-bottom:{l:?}"),
            Self::PaddingLeft(l) => format!("padding-left:{l:?}"),
            Self::PaddingRight(l) => format!("padding-right:{l:?}"),
            Self::MarginAll(l) => format!("margin:{l:?}"),
            Self::MarginX(l) => format!("margin-x:{l:?}"),
            Self::MarginY(l) => format!("margin-y:{l:?}"),
            Self::MarginTop(l) => format!("margin-top:{l:?}"),
            Self::MarginBottom(l) => format!("margin-bottom:{l:?}"),
            Self::MarginLeft(l) => format!("margin-left:{l:?}"),
            Self::MarginRight(l) => format!("margin-right:{l:?}"),
            Self::Position(p) => format!("position:{p:?}"),
            Self::Top(l) => format!("top:{l:?}"),
            Self::Right(l) => format!("right:{l:?}"),
            Self::Bottom(l) => format!("bottom:{l:?}"),
            Self::Left(l) => format!("left:{l:?}"),
            Self::Overflow(o) => format!("overflow:{o:?}"),
            Self::Background(c) => format!("background:{}", describe_col(c)),
            Self::BorderColor(c) => format!("border-color:{}", describe_col(c)),
            Self::BorderWidth(side, w) => format!("border-width:{side:?}:{w}"),
            Self::Rounded(r) => format!("rounded:{r}"),
            Self::RoundedCorner(c, r) => format!("rounded:{c:?}:{r}"),
            Self::Shadow(s) => format!("shadow:{s:?}"),
            Self::Opacity(o) => format!("opacity:{o}"),
            Self::Cursor(c) => format!("cursor:{c:?}"),
            Self::TextColor(c) => format!("text-color:{}", describe_col(c)),
            Self::TextSize(v) => format!("text-size:{v}"),
            Self::TextWeight(w) => format!("text-weight:{}", w.0),
            Self::TextAlign(a) => format!("text-align:{a:?}"),
            Self::WhitespaceNowrap => "whitespace-nowrap".to_owned(),
            Self::Truncate => "truncate".to_owned(),
            Self::FontFamily(f) => format!("font-family:{f}"),
            Self::SelectNone => "select-none".to_owned(),
        }
    }
}

/// Describes a color for [`Decl::describe`].
fn describe_col(col: &Col) -> String {
    match col {
        Col::Rgb(hex) => format!("rgb({:06x})", hex.value & 0xff_ffff),
        Col::Transparent => "transparent".to_owned(),
        Col::White => "white".to_owned(),
        Col::Black => "black".to_owned(),
        Col::Token(t) => format!("token:{}", t.fn_name()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_scale_round_trips() {
        for scale in [
            TextScale::Xs,
            TextScale::Sm,
            TextScale::Base,
            TextScale::Lg,
            TextScale::Xl,
            TextScale::Xxl,
            TextScale::Xxl3,
            TextScale::Xxl4,
        ] {
            assert_eq!(TextScale::from_px(scale.px()), Some(scale));
        }
        assert_eq!(TextScale::from_px(13.0), None);
    }

    #[test]
    fn weight_from_name_matches_named_classes() {
        assert_eq!(Weight::from_name(700), Some(Weight::BOLD));
        assert_eq!(Weight::from_name(500), Some(Weight::MEDIUM));
        assert_eq!(Weight::from_name(450), None);
    }

    #[test]
    fn spacing_scale_is_four_px_per_unit() {
        assert_eq!(Len::spacing(2.0), Len::Px(8.0));
        assert_eq!(Len::spacing(0.0), Len::Px(0.0));
    }

    #[test]
    fn inherited_text_marks_exactly_the_text_props() {
        assert!(Decl::TextColor(Col::Black).is_inherited_text());
        assert!(Decl::TextSize(14.0).is_inherited_text());
        assert!(Decl::TextWeight(Weight::BOLD).is_inherited_text());
        assert!(!Decl::Gap(Len::Px(8.0)).is_inherited_text());
        assert!(!Decl::Background(Col::Black).is_inherited_text());
    }

    #[test]
    fn token_names_are_snake_case() {
        assert_eq!(Token::MutedForeground.fn_name(), "muted_foreground");
        assert_eq!(Token::ALL.len(), 15);
    }
}
