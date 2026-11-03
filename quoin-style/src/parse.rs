//! Tailwind class-string → `Vec<Decl>` parser.
//!
//! [`parse_tailwind`] is the single entry point every backend uses. It
//! splits the class string on whitespace, separates `hover:`-prefixed
//! classes into their own list, and lowers each class into one [`Decl`].
//! Classes that do not parse are reported in [`ParsedClasses::unknown`];
//! backends turn those into `compile_error!` diagnostics (with
//! [`suggest_class`](crate::suggest_class) supplying the did-you-mean
//! suggestion) — silent dropping was the pre-IR behavior and is gone.

use crate::decl::{
    BorderSide, Col, Cursor, Decl, Display, FlexDir, HexLit, Items, Justify, Len, Overflow,
    Position, Shadow, TextAlign, TextScale, Token, Weight,
};

/// The result of parsing one `class="…"` attribute.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedClasses {
    /// Classes applied unconditionally.
    pub base: Vec<Decl>,
    /// Classes behind a `hover:` prefix.
    pub hover: Vec<Decl>,
    /// Classes that could not be recognized. Backends must surface these as
    /// compile errors; they are never dropped.
    pub unknown: Vec<String>,
}

impl ParsedClasses {
    /// True when nothing was unrecognized.
    #[must_use]
    pub fn is_known(&self) -> bool {
        self.unknown.is_empty()
    }
}

/// Every class the parser accepts, for did-you-mean suggestions and for the
/// coverage test that keeps this list in sync with [`parse_one`].
pub const KNOWN_CLASSES: &[&str] = &[
    // Display
    "flex",
    "inline-flex",
    "block",
    "inline-block",
    "hidden",
    // Flex direction & wrap
    "flex-col",
    "flex-row",
    "flex-col-reverse",
    "flex-row-reverse",
    "flex-wrap",
    "flex-nowrap",
    "grow",
    "flex-grow",
    "grow-0",
    "flex-grow-0",
    "shrink",
    "flex-shrink",
    "shrink-0",
    "flex-shrink-0",
    "flex-1",
    // Alignment
    "items-start",
    "items-end",
    "items-center",
    "items-baseline",
    "items-stretch",
    "justify-start",
    "justify-end",
    "justify-center",
    "justify-between",
    "justify-around",
    "justify-evenly",
    "self-start",
    "self-end",
    "self-center",
    // Position & overflow
    "absolute",
    "relative",
    "overflow-hidden",
    "overflow-auto",
    "overflow-scroll",
    "overflow-x-hidden",
    "overflow-y-hidden",
    "overflow-x-scroll",
    "overflow-y-scroll",
    "overflow-x-auto",
    "overflow-y-auto",
    // Size
    "w-full",
    "w-auto",
    "h-full",
    "h-auto",
    "size-full",
    "min-h-0",
    "min-w-0",
    "max-w-full",
    "max-h-full",
    // Opacity
    "opacity-0",
    "opacity-25",
    "opacity-50",
    "opacity-75",
    "opacity-100",
    // Background palette
    "bg-white",
    "bg-black",
    "bg-transparent",
    "bg-gray-100",
    "bg-gray-200",
    "bg-gray-300",
    "bg-gray-400",
    "bg-gray-500",
    "bg-gray-600",
    "bg-gray-700",
    "bg-gray-800",
    "bg-gray-900",
    "bg-gray-950",
    "bg-red-500",
    "bg-red-600",
    "bg-blue-500",
    "bg-blue-600",
    "bg-green-500",
    "bg-green-600",
    "bg-purple-500",
    "bg-purple-600",
    "bg-yellow-500",
    "bg-yellow-600",
    // Text palette
    "text-white",
    "text-black",
    "text-gray-100",
    "text-gray-200",
    "text-gray-300",
    "text-gray-400",
    "text-gray-500",
    "text-gray-600",
    "text-gray-700",
    "text-gray-800",
    "text-gray-900",
    "text-gray-950",
    "text-red-500",
    "text-red-600",
    "text-blue-500",
    "text-blue-600",
    "text-green-500",
    "text-green-600",
    "text-purple-500",
    "text-purple-600",
    "text-yellow-500",
    "text-yellow-600",
    // Theme tokens
    "text-primary",
    "text-secondary",
    "text-background",
    "text-foreground",
    "text-muted",
    "text-muted-foreground",
    "text-accent",
    "text-info",
    "text-warning",
    "text-danger",
    "text-destructive",
    "text-success",
    "text-border",
    "text-input",
    "text-ring",
    "text-card",
    "text-popover",
    "bg-primary",
    "bg-secondary",
    "bg-background",
    "bg-foreground",
    "bg-muted",
    "bg-muted-foreground",
    "bg-accent",
    "bg-border",
    "bg-card",
    "bg-popover",
    "bg-input",
    "bg-ring",
    "bg-info",
    "bg-warning",
    "bg-danger",
    "bg-destructive",
    "bg-success",
    // Typography
    "text-xs",
    "text-sm",
    "text-base",
    "text-lg",
    "text-xl",
    "text-2xl",
    "text-3xl",
    "text-4xl",
    "font-thin",
    "font-light",
    "font-normal",
    "font-medium",
    "font-semibold",
    "font-bold",
    "whitespace-nowrap",
    "truncate",
    "text-center",
    "text-left",
    "text-right",
    "select-none",
    "font-mono",
    "font-sans",
    // Borders
    "border",
    "border-0",
    "border-2",
    "border-gray-100",
    "border-gray-200",
    "border-gray-300",
    "border-gray-400",
    "border-gray-500",
    "border-gray-600",
    "border-gray-700",
    "border-gray-800",
    "border-gray-900",
    "border-gray-950",
    "border-red-500",
    "border-red-600",
    "border-blue-500",
    "border-blue-600",
    "border-green-500",
    "border-green-600",
    "border-purple-500",
    "border-purple-600",
    "border-yellow-500",
    "border-yellow-600",
    "border-white",
    "border-black",
    "border-b",
    "border-b-0",
    "border-b-2",
    "border-t",
    "border-t-0",
    "border-l",
    "border-r",
    "rounded-none",
    "rounded-sm",
    "rounded",
    "rounded-md",
    "rounded-lg",
    "rounded-xl",
    "rounded-2xl",
    "rounded-full",
    "rounded-tl",
    "rounded-tr",
    "rounded-bl",
    "rounded-br",
    "rounded-tl-lg",
    "rounded-tr-lg",
    "rounded-bl-lg",
    "rounded-br-lg",
    "rounded-tl-md",
    "rounded-tl-xl",
    // Shadows
    "shadow-xs",
    "shadow-sm",
    "shadow-md",
    "shadow-lg",
    "shadow-xl",
    // Cursor
    "cursor-pointer",
    "cursor-default",
    "cursor-text",
    // Spacing scales (each numeric step is one class)
    "gap-0",
    "gap-1",
    "gap-1p5",
    "gap-2",
    "gap-3",
    "gap-4",
    "gap-5",
    "gap-6",
    "gap-8",
    "gap-10",
    "gap-12",
    "p-0",
    "p-1",
    "p-2",
    "p-3",
    "p-4",
    "p-5",
    "p-6",
    "p-8",
    "p-10",
    "px-0",
    "px-1",
    "px-2",
    "px-3",
    "px-4",
    "px-5",
    "px-6",
    "px-8",
    "px-1p5",
    "py-0",
    "py-1",
    "py-2",
    "py-3",
    "py-4",
    "py-5",
    "py-6",
    "py-8",
    "py-1p5",
    "m-0",
    "m-1",
    "m-2",
    "m-4",
    "mx-auto",
    "mx-0",
    "mx-1",
    "mx-2",
    "mx-3",
    "mx-4",
    "my-0",
    "my-1",
    "my-2",
    "my-3",
    "my-4",
    "mt-0",
    "mt-1",
    "mt-2",
    "mt-3",
    "mt-4",
    "mb-0",
    "mb-1",
    "mb-2",
    "mb-3",
    "mb-4",
    "ml-0",
    "ml-1",
    "ml-2",
    "ml-3",
    "ml-auto",
    "mr-0",
    "mr-1",
    "mr-2",
    "mr-3",
    "mr-auto",
    "pl-0",
    "pl-1",
    "pl-2",
    "pl-3",
    "pl-4",
    "pr-0",
    "pr-1",
    "pr-2",
    "pr-3",
    "pr-4",
    "pt-0",
    "pt-1",
    "pt-2",
    "pt-3",
    "pt-4",
    "pb-0",
    "pb-1",
    "pb-2",
    "pb-3",
    "pb-4",
    // Absolute offsets
    "top-0",
    "top-1",
    "top-2",
    "top-3",
    "right-0",
    "right-1",
    "right-2",
    "right-3",
    "bottom-0",
    "bottom-1",
    "bottom-2",
    "bottom-3",
    "left-0",
    "left-1",
    "left-2",
    "left-3",
];

/// Parses a Tailwind class string into typed decls.
///
/// `hover:`-prefixed classes land in [`ParsedClasses::hover`]; everything
/// else in [`ParsedClasses::base`]; unrecognized classes in
/// [`ParsedClasses::unknown`].
#[must_use]
pub fn parse_tailwind(class_str: &str) -> ParsedClasses {
    let mut out = ParsedClasses::default();
    for class in class_str.split_whitespace() {
        if let Some(rest) = class.strip_prefix("hover:") {
            match parse_one(rest) {
                Some(decl) => out.hover.push(decl),
                None => out.unknown.push(rest.to_owned()),
            }
        } else {
            match parse_one(class) {
                Some(decl) => out.base.push(decl),
                None => out.unknown.push(class.to_owned()),
            }
        }
    }
    out
}

/// Parses `rrggbb` or `rrggbbaa` (no `#` prefix) into a hex literal (alpha
/// 0-255).
fn hex_color(inner: &str) -> Option<(HexLit, f32)> {
    let (rgb_text, alpha) = match inner.len() {
        6 => (inner, 1.0),
        8 => (
            inner.get(..6)?,
            f32::from(u8::from_str_radix(inner.get(6..)?, 16).ok()?) / 255.0,
        ),
        _ => return None,
    };
    let value = u32::from_str_radix(rgb_text, 16).ok()?;
    Some((
        HexLit {
            text: "",
            value: value << 8 | opaque_alpha(alpha),
        },
        alpha,
    ))
}

/// Packs an alpha fraction into the high byte of an RGBA literal.
fn opaque_alpha(alpha: f32) -> u32 {
    if alpha >= 1.0 {
        0xff
    } else {
        // Truncation matches the pre-IR GPUI transpiler.
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = (alpha * 255.0).round() as u32;
        byte
    }
}

/// Parses an arbitrary value (the part between the brackets) as a px length.
fn arbitrary_px(inner: &str) -> Option<f32> {
    let value = inner.strip_suffix("px").unwrap_or(inner);
    value.parse::<f32>().ok()
}

/// Parses one class (without any `hover:` prefix) into a [`Decl`].
///
/// Returns `None` for classes outside the vocabulary; the caller records
/// them as unknown so backends can emit diagnostics.
#[must_use]
pub fn parse_one(class: &str) -> Option<Decl> {
    layout(class)
        .or_else(|| spacing(class))
        .or_else(|| color(class))
        .or_else(|| typography(class))
        .or_else(|| border(class))
        .or_else(|| misc(class))
        .or_else(|| arbitrary(class))
        .or_else(|| token_class(class))
}

fn spacing_units(class: &str, prefix: &str) -> Option<Len> {
    let rest = class.strip_prefix(prefix)?;
    if rest == "auto" {
        Some(Len::Auto)
    } else {
        let units = match rest {
            "0" => 0.0,
            "1" => 1.0,
            "1p5" => 1.5,
            "2" => 2.0,
            "3" => 3.0,
            "4" => 4.0,
            "5" => 5.0,
            "6" => 6.0,
            "8" => 8.0,
            "10" => 10.0,
            "12" => 12.0,
            // Tailwind v4 generates the spacing scale dynamically; any
            // non-negative integer is a valid multiple of 4 px.
            _ => rest
                .parse::<i64>()
                .ok()
                .filter(|n| *n >= 0)
                .map(i64_to_px_f32)?,
        };
        Some(Len::spacing(units))
    }
}

/// Non-negative Tailwind pixel steps fit `f32` exactly (multiples of 4 px
/// below 2^24); the cast is lossless for every vocabulary value.
#[expect(clippy::cast_precision_loss)]
fn i64_to_px_f32(n: i64) -> f32 {
    n as f32
}

fn layout(class: &str) -> Option<Decl> {
    Some(match class {
        // Display
        "flex" => Decl::Display(Display::Flex),
        "inline-flex" => Decl::Display(Display::InlineFlex),
        "block" => Decl::Display(Display::Block),
        "inline-block" => Decl::Display(Display::InlineBlock),
        "hidden" => Decl::Display(Display::Hidden),
        // Flex direction
        "flex-col" => Decl::FlexDir(FlexDir::Column),
        "flex-row" => Decl::FlexDir(FlexDir::Row),
        "flex-col-reverse" => Decl::FlexDir(FlexDir::ColumnReverse),
        "flex-row-reverse" => Decl::FlexDir(FlexDir::RowReverse),
        // Flex wrap
        "flex-wrap" => Decl::FlexWrap(true),
        "flex-nowrap" => Decl::FlexWrap(false),
        // Grow / shrink
        "flex-grow" | "grow" => Decl::Grow(1.0),
        "flex-grow-0" | "grow-0" => Decl::Grow(0.0),
        "flex-shrink" | "shrink" => Decl::Shrink(1.0),
        "flex-shrink-0" | "shrink-0" => Decl::Shrink(0.0),
        "flex-1" => Decl::Flex1,
        // Alignment
        "items-start" => Decl::Items(Items::Start),
        "items-end" => Decl::Items(Items::End),
        "items-center" => Decl::Items(Items::Center),
        "items-baseline" => Decl::Items(Items::Baseline),
        "items-stretch" => Decl::Items(Items::Stretch),
        "justify-start" => Decl::Justify(Justify::Start),
        "justify-end" => Decl::Justify(Justify::End),
        "justify-center" => Decl::Justify(Justify::Center),
        "justify-between" => Decl::Justify(Justify::Between),
        "justify-around" => Decl::Justify(Justify::Around),
        "justify-evenly" => Decl::Justify(Justify::Evenly),
        "self-start" => Decl::SelfAlign(Items::Start),
        "self-end" => Decl::SelfAlign(Items::End),
        "self-center" => Decl::SelfAlign(Items::Center),
        // Position
        "absolute" => Decl::Position(Position::Absolute),
        "relative" => Decl::Position(Position::Relative),
        // Overflow
        "overflow-hidden" | "overflow-x-hidden" | "overflow-y-hidden" => {
            Decl::Overflow(Overflow::Hidden)
        }
        "overflow-auto" => Decl::Overflow(Overflow::AutoY),
        "overflow-scroll" => Decl::Overflow(Overflow::Scroll),
        "overflow-y-scroll" | "overflow-y-auto" => Decl::Overflow(Overflow::ScrollY),
        "overflow-x-scroll" | "overflow-x-auto" => Decl::Overflow(Overflow::ScrollX),
        // Size
        "w-full" => Decl::Width(Len::Full),
        "w-auto" => Decl::Width(Len::Auto),
        "h-full" => Decl::Height(Len::Full),
        "h-auto" => Decl::Height(Len::Auto),
        "size-full" => Decl::SizeFull,
        "min-h-0" => Decl::MinHeight(Len::Px(0.0)),
        "min-w-0" => Decl::MinWidth(Len::Px(0.0)),
        "max-w-full" => Decl::MaxWidth(Len::Full),
        "max-h-full" => Decl::MaxHeight(Len::Full),
        _ => return None,
    })
}

fn spacing(class: &str) -> Option<Decl> {
    Some(match class {
        _ if class.starts_with("gap-") => Decl::Gap(spacing_units(class, "gap-")?),
        _ if class.starts_with("p-") => Decl::PaddingAll(spacing_units(class, "p-")?),
        _ if class.starts_with("px-") => Decl::PaddingX(spacing_units(class, "px-")?),
        _ if class.starts_with("py-") => Decl::PaddingY(spacing_units(class, "py-")?),
        _ if class.starts_with("pt-") => Decl::PaddingTop(spacing_units(class, "pt-")?),
        _ if class.starts_with("pb-") => Decl::PaddingBottom(spacing_units(class, "pb-")?),
        _ if class.starts_with("pl-") => Decl::PaddingLeft(spacing_units(class, "pl-")?),
        _ if class.starts_with("pr-") => Decl::PaddingRight(spacing_units(class, "pr-")?),
        _ if class.starts_with("m-") => Decl::MarginAll(spacing_units(class, "m-")?),
        _ if class.starts_with("mx-") => Decl::MarginX(spacing_units(class, "mx-")?),
        _ if class.starts_with("my-") => Decl::MarginY(spacing_units(class, "my-")?),
        _ if class.starts_with("mt-") => Decl::MarginTop(spacing_units(class, "mt-")?),
        _ if class.starts_with("mb-") => Decl::MarginBottom(spacing_units(class, "mb-")?),
        _ if class.starts_with("ml-") => Decl::MarginLeft(spacing_units(class, "ml-")?),
        _ if class.starts_with("mr-") => Decl::MarginRight(spacing_units(class, "mr-")?),
        _ if class.starts_with("top-") => Decl::Top(spacing_units(class, "top-")?),
        _ if class.starts_with("right-") => Decl::Right(spacing_units(class, "right-")?),
        _ if class.starts_with("bottom-") => Decl::Bottom(spacing_units(class, "bottom-")?),
        _ if class.starts_with("left-") => Decl::Left(spacing_units(class, "left-")?),
        // Widths and heights ride the same 4-px-per-unit scale.
        _ if class.starts_with("min-w-") => Decl::MinWidth(spacing_units(class, "min-w-")?),
        _ if class.starts_with("min-h-") => Decl::MinHeight(spacing_units(class, "min-h-")?),
        _ if class.starts_with("max-w-") => Decl::MaxWidth(spacing_units(class, "max-w-")?),
        _ if class.starts_with("max-h-") => Decl::MaxHeight(spacing_units(class, "max-h-")?),
        _ if class.starts_with("w-") => Decl::Width(spacing_units(class, "w-")?),
        _ if class.starts_with("h-") => Decl::Height(spacing_units(class, "h-")?),
        _ => return None,
    })
}

/// Gray/blue/red/green/purple/yellow palette colors, shared by the
/// background, text and border tables.
fn palette_hex(class: &str) -> Option<HexLit> {
    Some(match class {
        "white" => HexLit::new("ffffff", 0xff_ff_ff_ff),
        "black" => HexLit::new("000000", 0x00_00_00_ff),
        "gray-100" => HexLit::new("f3f4f6", 0xf3_f4_f6_ff),
        "gray-200" => HexLit::new("e5e7eb", 0xe5_e7_eb_ff),
        "gray-300" => HexLit::new("d1d5db", 0xd1_d5_db_ff),
        "gray-400" => HexLit::new("9ca3af", 0x9c_a3_af_ff),
        "gray-500" => HexLit::new("6b7280", 0x6b_72_80_ff),
        "gray-600" => HexLit::new("4b5563", 0x4b_55_63_ff),
        "gray-700" => HexLit::new("374151", 0x37_41_51_ff),
        "gray-800" => HexLit::new("1f2937", 0x1f_29_37_ff),
        "gray-900" => HexLit::new("111827", 0x11_18_27_ff),
        "gray-950" => HexLit::new("030712", 0x03_07_12_ff),
        "red-500" => HexLit::new("ef4444", 0xef_44_44_ff),
        "red-600" => HexLit::new("dc2626", 0xdc_26_26_ff),
        "blue-500" => HexLit::new("3b82f6", 0x3b_82_f6_ff),
        "blue-600" => HexLit::new("2563eb", 0x25_63_eb_ff),
        "green-500" => HexLit::new("22c55e", 0x22_c5_5e_ff),
        "green-600" => HexLit::new("16a34a", 0x16_a3_4a_ff),
        "purple-500" => HexLit::new("a855f7", 0xa8_55_f7_ff),
        "purple-600" => HexLit::new("9333ea", 0x93_33_ea_ff),
        "yellow-500" => HexLit::new("eab308", 0xea_b3_08_ff),
        "yellow-600" => HexLit::new("ca8a04", 0xca_8a_04_ff),
        _ => return None,
    })
}

fn color(class: &str) -> Option<Decl> {
    if let Some(opacity) = class.strip_prefix("opacity-") {
        return match opacity {
            "0" => Some(Decl::Opacity(0.0)),
            "25" => Some(Decl::Opacity(0.25)),
            "50" => Some(Decl::Opacity(0.5)),
            "75" => Some(Decl::Opacity(0.75)),
            "100" => Some(Decl::Opacity(1.0)),
            _ => None,
        };
    }
    if let Some(rest) = class.strip_prefix("bg-") {
        if rest == "transparent" {
            return Some(Decl::Background(Col::Transparent));
        }
        if rest == "white" {
            return Some(Decl::Background(Col::White));
        }
        if rest == "black" {
            return Some(Decl::Background(Col::Black));
        }
        return palette_hex(rest).map(|hex| Decl::Background(Col::Rgb(hex)));
    }
    if let Some(rest) = class.strip_prefix("text-") {
        if rest == "white" {
            return Some(Decl::TextColor(Col::White));
        }
        if rest == "black" {
            return Some(Decl::TextColor(Col::Black));
        }
        return palette_hex(rest).map(|hex| Decl::TextColor(Col::Rgb(hex)));
    }
    if let Some(rest) = class.strip_prefix("border-") {
        if rest == "white" {
            return Some(Decl::BorderColor(Col::White));
        }
        if rest == "black" {
            return Some(Decl::BorderColor(Col::Black));
        }
        return palette_hex(rest).map(|hex| Decl::BorderColor(Col::Rgb(hex)));
    }
    None
}

fn typography(class: &str) -> Option<Decl> {
    Some(match class {
        "text-xs" => Decl::TextSize(TextScale::Xs.px()),
        "text-sm" => Decl::TextSize(TextScale::Sm.px()),
        "text-base" => Decl::TextSize(TextScale::Base.px()),
        "text-lg" => Decl::TextSize(TextScale::Lg.px()),
        "text-xl" => Decl::TextSize(TextScale::Xl.px()),
        "text-2xl" => Decl::TextSize(TextScale::Xxl.px()),
        "text-3xl" => Decl::TextSize(TextScale::Xxl3.px()),
        "text-4xl" => Decl::TextSize(TextScale::Xxl4.px()),
        "font-thin" => Decl::TextWeight(Weight::THIN),
        "font-light" => Decl::TextWeight(Weight::LIGHT),
        "font-normal" => Decl::TextWeight(Weight::NORMAL),
        "font-medium" => Decl::TextWeight(Weight::MEDIUM),
        "font-semibold" => Decl::TextWeight(Weight::SEMIBOLD),
        "font-bold" => Decl::TextWeight(Weight::BOLD),
        "whitespace-nowrap" => Decl::WhitespaceNowrap,
        "truncate" => Decl::Truncate,
        "text-center" => Decl::TextAlign(TextAlign::Center),
        "text-left" => Decl::TextAlign(TextAlign::Left),
        "text-right" => Decl::TextAlign(TextAlign::Right),
        "select-none" => Decl::SelectNone,
        "font-mono" => Decl::FontFamily("monospace"),
        "font-sans" => Decl::FontFamily("sans-serif"),
        _ => return None,
    })
}

fn border(class: &str) -> Option<Decl> {
    Some(match class {
        "border" => Decl::BorderWidth(BorderSide::All, 1.0),
        "border-0" => Decl::BorderWidth(BorderSide::All, 0.0),
        "border-2" => Decl::BorderWidth(BorderSide::All, 2.0),
        "border-b" => Decl::BorderWidth(BorderSide::Bottom, 1.0),
        "border-b-0" => Decl::BorderWidth(BorderSide::Bottom, 0.0),
        "border-b-2" => Decl::BorderWidth(BorderSide::Bottom, 2.0),
        "border-t" => Decl::BorderWidth(BorderSide::Top, 1.0),
        "border-t-0" => Decl::BorderWidth(BorderSide::Top, 0.0),
        "border-l" => Decl::BorderWidth(BorderSide::Left, 1.0),
        "border-r" => Decl::BorderWidth(BorderSide::Right, 1.0),
        "rounded-none" => Decl::Rounded(0.0),
        "rounded-sm" => Decl::Rounded(2.0),
        "rounded" => Decl::Rounded(4.0),
        "rounded-md" => Decl::Rounded(6.0),
        "rounded-lg" => Decl::Rounded(8.0),
        "rounded-xl" => Decl::Rounded(12.0),
        "rounded-2xl" => Decl::Rounded(16.0),
        "rounded-full" => Decl::Rounded(9999.0),
        "rounded-tl" => Decl::RoundedCorner(crate::decl::Corner::TopLeft, 4.0),
        "rounded-tr" => Decl::RoundedCorner(crate::decl::Corner::TopRight, 4.0),
        "rounded-bl" => Decl::RoundedCorner(crate::decl::Corner::BottomLeft, 4.0),
        "rounded-br" => Decl::RoundedCorner(crate::decl::Corner::BottomRight, 4.0),
        "rounded-tl-lg" => Decl::RoundedCorner(crate::decl::Corner::TopLeft, 8.0),
        "rounded-tr-lg" => Decl::RoundedCorner(crate::decl::Corner::TopRight, 8.0),
        "rounded-bl-lg" => Decl::RoundedCorner(crate::decl::Corner::BottomLeft, 8.0),
        "rounded-br-lg" => Decl::RoundedCorner(crate::decl::Corner::BottomRight, 8.0),
        "rounded-tl-md" => Decl::RoundedCorner(crate::decl::Corner::TopLeft, 6.0),
        "rounded-tl-xl" => Decl::RoundedCorner(crate::decl::Corner::TopLeft, 12.0),
        "shadow-xs" => Decl::Shadow(Shadow::Xs),
        "shadow-sm" => Decl::Shadow(Shadow::Sm),
        "shadow-md" => Decl::Shadow(Shadow::Md),
        "shadow-lg" => Decl::Shadow(Shadow::Lg),
        "shadow-xl" => Decl::Shadow(Shadow::Xl),
        _ => return None,
    })
}

fn misc(class: &str) -> Option<Decl> {
    Some(match class {
        "cursor-pointer" => Decl::Cursor(Cursor::Pointer),
        "cursor-default" => Decl::Cursor(Cursor::Default),
        "cursor-text" => Decl::Cursor(Cursor::Text),
        _ => return None,
    })
}

/// Arbitrary-value classes: `p-[13px]`, `w-[600px]`, `text-[11px]`,
/// `opacity-[0.9]`, `rounded-[10px]`, `bg-[#1e2e3e]`, … The `px` suffix is
/// optional so bare numbers work too.
fn arbitrary(class: &str) -> Option<Decl> {
    let (name, inner) = class.split_once('[')?;
    // `split_once` keeps the role's trailing dash (`bg-[#…]` → `"bg-"`), so
    // normalize before matching. (The pre-IR GPUI transpiler matched the
    // dashed name and therefore silently dropped every arbitrary value;
    // here they parse.)
    let name = name.strip_suffix('-').unwrap_or(name);
    let inner = inner.strip_suffix(']')?;
    if let Some(hex) = inner.strip_prefix('#') {
        let (lit, _alpha) = hex_color(hex)?;
        return match name {
            "bg" => Some(Decl::Background(Col::Rgb(lit))),
            "text" => Some(Decl::TextColor(Col::Rgb(lit))),
            "border" => Some(Decl::BorderColor(Col::Rgb(lit))),
            _ => None,
        };
    }
    let px = arbitrary_px(inner)?;
    Some(match name {
        "p" => Decl::PaddingAll(Len::Px(px)),
        "px" => Decl::PaddingX(Len::Px(px)),
        "py" => Decl::PaddingY(Len::Px(px)),
        "pt" => Decl::PaddingTop(Len::Px(px)),
        "pb" => Decl::PaddingBottom(Len::Px(px)),
        "pl" => Decl::PaddingLeft(Len::Px(px)),
        "pr" => Decl::PaddingRight(Len::Px(px)),
        "m" => Decl::MarginAll(Len::Px(px)),
        "mx" => Decl::MarginX(Len::Px(px)),
        "my" => Decl::MarginY(Len::Px(px)),
        "mt" => Decl::MarginTop(Len::Px(px)),
        "mb" => Decl::MarginBottom(Len::Px(px)),
        "ml" => Decl::MarginLeft(Len::Px(px)),
        "mr" => Decl::MarginRight(Len::Px(px)),
        "gap" => Decl::Gap(Len::Px(px)),
        "top" => Decl::Top(Len::Px(px)),
        "right" => Decl::Right(Len::Px(px)),
        "bottom" => Decl::Bottom(Len::Px(px)),
        "left" => Decl::Left(Len::Px(px)),
        "w" => Decl::Width(Len::Px(px)),
        "h" => Decl::Height(Len::Px(px)),
        "min-w" => Decl::MinWidth(Len::Px(px)),
        "min-h" => Decl::MinHeight(Len::Px(px)),
        "max-w" => Decl::MaxWidth(Len::Px(px)),
        "max-h" => Decl::MaxHeight(Len::Px(px)),
        "rounded" => Decl::Rounded(px),
        "rounded-tl" => Decl::RoundedCorner(crate::decl::Corner::TopLeft, px),
        "text" => Decl::TextSize(px),
        "opacity" => Decl::Opacity(px),
        _ => return None,
    })
}

/// Theme-token classes (`bg-primary`, `text-muted-foreground`, …).
///
/// The token is stored symbolically: the GPUI lowering bakes in the default
/// shadcn-dark palette (see [`crate::decl::Token::surface_hex`] /
/// [`crate::decl::Token::text_hex`]) to preserve its pre-IR output, while
/// the Floem and Xilem lowerings call their adapter's live theme functions.
fn token_class(class: &str) -> Option<Decl> {
    let (role, rest) = class.split_once('-')?;
    let token = match rest {
        "primary" => Token::Primary,
        "secondary" => Token::Secondary,
        "background" => Token::Background,
        "foreground" => Token::Foreground,
        "muted" => Token::Muted,
        "muted-foreground" => Token::MutedForeground,
        "accent" => Token::Accent,
        "info" => Token::Info,
        "warning" => Token::Warning,
        "danger" | "destructive" => Token::Danger,
        "success" => Token::Success,
        "border" => Token::Border,
        "input" => Token::Input,
        "ring" => Token::Ring,
        "card" | "popover" => Token::Card,
        _ => return None,
    };
    match role {
        "bg" => Some(Decl::Background(Col::Token(token))),
        "text" => Some(Decl::TextColor(Col::Token(token))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggest_class;

    #[test]
    fn every_known_class_parses() {
        for class in KNOWN_CLASSES {
            assert!(
                parse_one(class).is_some(),
                "KNOWN_CLASSES lists {class:?} but parse_one rejects it"
            );
        }
    }

    #[test]
    fn known_classes_list_has_no_duplicates() {
        let mut seen = std::collections::BTreeSet::new();
        for class in KNOWN_CLASSES {
            assert!(
                seen.insert(*class),
                "duplicate entry in KNOWN_CLASSES: {class}"
            );
        }
    }

    /// Suffix vocabulary shared by the `bg-`/`text-`/`border-` color roles.
    const COLOR_SUFFIXES: &[&str] = &[
        "white",
        "black",
        "gray-100",
        "gray-200",
        "gray-300",
        "gray-400",
        "gray-500",
        "gray-600",
        "gray-700",
        "gray-800",
        "gray-900",
        "gray-950",
        "red-500",
        "red-600",
        "blue-500",
        "blue-600",
        "green-500",
        "green-600",
        "purple-500",
        "purple-600",
        "yellow-500",
        "yellow-600",
        "primary",
        "secondary",
        "background",
        "foreground",
        "muted",
        "muted-foreground",
        "accent",
        "info",
        "success",
        "warning",
        "danger",
        "destructive",
        "border",
        "input",
        "ring",
        "card",
        "popover",
    ];

    #[test]
    fn every_parseable_color_class_is_listed() {
        for prefix in ["bg-", "text-", "border-"] {
            for suffix in COLOR_SUFFIXES {
                let class = format!("{prefix}{suffix}");
                if parse_one(&class).is_some() {
                    assert!(
                        KNOWN_CLASSES.contains(&class.as_str()),
                        "parse_one accepts {class:?} but KNOWN_CLASSES does not list it"
                    );
                } else {
                    // Roles without a table entry for this suffix (e.g.
                    // `border-input`) must stay out of KNOWN_CLASSES so the
                    // did-you-mean suggestions never point at them.
                    assert!(
                        !KNOWN_CLASSES.contains(&class.as_str()),
                        "KNOWN_CLASSES lists {class:?} but parse_one rejects it"
                    );
                }
            }
        }
        assert!(parse_one("bg-transparent").is_some());
        assert!(KNOWN_CLASSES.contains(&"bg-transparent"));
    }

    #[test]
    fn splits_hover_state() {
        let parsed = parse_tailwind("flex gap-2 hover:bg-primary hover:cursor-pointer");
        assert_eq!(parsed.base.len(), 2);
        assert_eq!(parsed.hover.len(), 2);
        assert!(parsed.is_known());
    }

    #[test]
    fn reports_unknown_classes() {
        let parsed = parse_tailwind("flex flec-row");
        assert_eq!(parsed.base.len(), 1);
        assert_eq!(parsed.unknown, vec!["flec-row".to_owned()]);
        assert_eq!(suggest_class("flec-row"), Some("flex-row"));
    }

    #[test]
    fn arbitrary_values_parse() {
        assert_eq!(parse_one("w-[600px]"), Some(Decl::Width(Len::Px(600.0))));
        assert_eq!(parse_one("text-[11px]"), Some(Decl::TextSize(11.0)));
        assert_eq!(parse_one("gap-[6]"), Some(Decl::Gap(Len::Px(6.0))));
        assert_eq!(parse_one("opacity-[0.9]"), Some(Decl::Opacity(0.9)));
    }

    #[test]
    fn arbitrary_hex_colors_parse() {
        assert!(matches!(
            parse_one("bg-[#1e2e3e]"),
            Some(Decl::Background(Col::Rgb(HexLit {
                value: 0x1e_2e_3e_ff,
                ..
            })))
        ));
        assert!(parse_one("text-#bad").is_none());
    }

    #[test]
    fn token_classes_are_symbolic() {
        assert_eq!(
            parse_one("bg-primary"),
            Some(Decl::Background(Col::Token(Token::Primary)))
        );
        assert_eq!(
            parse_one("text-muted-foreground"),
            Some(Decl::TextColor(Col::Token(Token::MutedForeground)))
        );
        // `destructive` is an alias of the danger token.
        assert_eq!(
            parse_one("bg-destructive"),
            Some(Decl::Background(Col::Token(Token::Danger)))
        );
        // Card and popover share the card surface token.
        assert_eq!(
            parse_one("bg-popover"),
            Some(Decl::Background(Col::Token(Token::Card)))
        );
    }

    #[test]
    fn spacing_scales_match_the_pre_ir_transpiler() {
        assert_eq!(parse_one("gap-2"), Some(Decl::Gap(Len::Px(8.0))));
        assert_eq!(parse_one("p-4"), Some(Decl::PaddingAll(Len::Px(16.0))));
        assert_eq!(parse_one("gap-1p5"), Some(Decl::Gap(Len::Px(6.0))));
        assert_eq!(parse_one("p-10"), Some(Decl::PaddingAll(Len::Px(40.0))));
        assert_eq!(parse_one("mx-auto"), Some(Decl::MarginX(Len::Auto)));
    }

    #[test]
    fn eight_digit_hex_carries_alpha() {
        let (lit, alpha) = hex_color("11223380").expect("parses");
        assert_eq!(alpha, 128.0 / 255.0);
        assert_eq!(lit.alpha(), 128);
        assert_eq!(lit.red(), 0x11);
    }
}
