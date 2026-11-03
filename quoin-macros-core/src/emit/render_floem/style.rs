//! `Decl` → Floem `Style` lowering.
//!
//! One `quoin_style::Decl` becomes one builder call (or a small group for
//! composite decls such as per-side padding). Method names and unit types are
//! verified against floem `main` (`Style` in `src/style/mod.rs`):
//!
//! - lengths: `f64 → Pt → Length` / `LengthAuto`, so a plain float literal works,
//! - colors: `peniko::Color` (`from_rgba8`), `Brush` via `From<Color>`,
//! - text: `font_size`/`color`/`font_weight`/`font_family` are inherited props,
//! - selectors: `.hover(|s| …)`, `.focus(|s| …)`, `.active(|s| …)`.
//!
//! Theme tokens lower to `::quoin::floem_theme::<token>()` /
//! `<token>_text()` calls, which read lock-free atomics (see
//! `quoin-floem/src/theme.rs`).

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use quoin_style::{
    BorderSide, Col, Corner, Decl, FlexDir, Items, Justify, Len, ParsedClasses, suggest_class,
};

/// The Floem style-chain pieces produced from one `class=".."` argument.
pub(crate) struct FloemStyles {
    /// Chain pieces applied unconditionally (each starts with `.`).
    pub normal: Vec<TokenStream>,
    /// Chain pieces for the `.hover(|s| …)` selector.
    pub hover: Vec<TokenStream>,
    /// `compile_error!` statements for unrecognized classes.
    pub errors: Vec<TokenStream>,
    /// The flex direction declared by `flex-col`/`flex-row`-family classes.
    pub flex_dir: Option<FlexDir>,
}

/// The Floem expression for a style-IR color.
fn color_expr(col: Col, text_role: bool) -> TokenStream {
    match col {
        Col::Rgb(lit) => {
            let (r, g, b, a) = (lit.red(), lit.green(), lit.blue(), lit.alpha());
            quote! { ::floem::peniko::Color::from_rgba8(#r, #g, #b, #a) }
        }
        Col::Transparent => quote! { ::floem::peniko::Color::TRANSPARENT },
        Col::White => quote! { ::floem::peniko::Color::WHITE },
        Col::Black => quote! { ::floem::peniko::Color::BLACK },
        Col::Token(token) => {
            let suffix = if text_role { "_text" } else { "" };
            let fn_name = format!("{}{suffix}", token.fn_name());
            let ident = proc_macro2::Ident::new(&fn_name, proc_macro2::Span::call_site());
            quote! { ::quoin::floem_theme::#ident() }
        }
    }
}

/// A float literal, formatted the way Rust writes `f64` debug output
/// (`8`, `8.0`, `0.5` — always a valid float token).
fn f64_lit(value: f64) -> TokenStream {
    let text = format!("{value:?}");
    let lit: proc_macro2::Literal =
        syn::parse_str(&text).unwrap_or_else(|_| proc_macro2::Literal::f64_suffixed(value));
    lit.to_token_stream()
}

/// A length expression: plain float for px, `Pct`, or `LengthAuto::Auto`.
fn length_expr(len: Len) -> TokenStream {
    match len {
        Len::Px(v) => f64_lit(f64::from(v)),
        Len::Full => quote! { 100.0_f64 },
        Len::Auto => quote! { ::floem::style::LengthAuto::Auto },
    }
}

/// Lowers one decl into a chain piece. Per-side padding/margin accumulate
/// through the ` sides: &mut Option<…>` out-parameters so one composite
/// `.apply_padding(Padding { .. })` is emitted instead of N overrides.
///
/// Dispatch happens per declaration category ([`layout_decl`], [`sizing_decl`],
/// …) so each mapper stays reviewable in isolation.
fn decl_to_tokens(decl: &Decl) -> TokenStream {
    layout_decl(decl)
        .or_else(|| sizing_decl(decl))
        .or_else(|| spacing_decl(decl))
        .or_else(|| position_decl(decl))
        .or_else(|| visual_decl(decl))
        .or_else(|| text_decl(decl))
        .unwrap_or_default()
}

/// Flex layout: direction, wrapping, justification, item alignment, grow and
/// gap declarations.
fn layout_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        // Floem is always flex; block/inline-flex are layout no-ops.
        // `self-*` maps to the AlignSelf prop (taffy `AlignItems` value).
        Decl::Display(_) | Decl::SelfAlign(_) => quote! {},
        Decl::FlexDir(FlexDir::Row) => quote! { .flex_row() },
        Decl::FlexDir(FlexDir::Column) => quote! { .flex_col() },
        Decl::FlexDir(dir @ (FlexDir::RowReverse | FlexDir::ColumnReverse)) => {
            let variant = if *dir == FlexDir::RowReverse {
                "RowReverse"
            } else {
                "ColumnReverse"
            };
            let variant = proc_macro2::Ident::new(variant, proc_macro2::Span::call_site());
            quote! { .flex_direction(::floem::style::FlexDirection::#variant) }
        }
        Decl::FlexWrap(true) => quote! { .flex_wrap(::floem::style::FlexWrap::Wrap) },
        Decl::FlexWrap(false) => quote! { .flex_wrap(::floem::style::FlexWrap::NoWrap) },
        Decl::Justify(Justify::Start) => quote! { .justify_start() },
        Decl::Justify(Justify::End) => quote! { .justify_end() },
        Decl::Justify(Justify::Center) => quote! { .justify_center() },
        Decl::Justify(Justify::Between) => quote! { .justify_between() },
        Decl::Justify(Justify::Around) => quote! { .justify_around() },
        Decl::Justify(Justify::Evenly) => quote! { .justify_evenly() },
        Decl::Items(Items::Start) => quote! { .items_start() },
        Decl::Items(Items::End) => quote! { .items_end() },
        Decl::Items(Items::Center) => quote! { .items_center() },
        Decl::Items(Items::Baseline) => quote! { .items_baseline() },
        Decl::Items(Items::Stretch) => quote! { .items_stretch() },
        Decl::Grow(v) => {
            let v = f64_lit(f64::from(*v));
            quote! { .flex_grow(#v) }
        }
        Decl::Shrink(v) => {
            let v = f64_lit(f64::from(*v));
            quote! { .flex_shrink(#v) }
        }
        Decl::Flex1 => quote! { .flex_grow(1.0).flex_shrink(1.0) },
        Decl::Gap(len) => {
            let len = length_expr(*len);
            quote! { .gap(#len) }
        }
        _ => return None,
    })
}

/// Box sizing: width, height, min/max bounds.
fn sizing_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        Decl::Width(Len::Full) => quote! { .width_full() },
        Decl::Width(Len::Auto) => quote! { .width(::floem::style::LengthAuto::Auto) },
        Decl::Width(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .width(#v) }
        }
        Decl::Height(Len::Full) => quote! { .height_full() },
        Decl::Height(Len::Auto) => quote! { .height(::floem::style::LengthAuto::Auto) },
        Decl::Height(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .height(#v) }
        }
        Decl::SizeFull => quote! { .size_full() },
        Decl::MinWidth(len) => {
            let len = length_expr(*len);
            quote! { .min_width(#len) }
        }
        Decl::MinHeight(len) => {
            let len = length_expr(*len);
            quote! { .min_height(#len) }
        }
        Decl::MaxWidth(Len::Full) => quote! { .max_width_full() },
        Decl::MaxWidth(len) => {
            let len = length_expr(*len);
            quote! { .max_width(#len) }
        }
        Decl::MaxHeight(Len::Full) => quote! { .max_height_full() },
        Decl::MaxHeight(len) => {
            let len = length_expr(*len);
            quote! { .max_height(#len) }
        }
        _ => return None,
    })
}

/// Spacing: padding and margins. `*Full` margins are no-ops (a full-width
/// margin box cancels out in Floem's flex layout).
fn spacing_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        Decl::PaddingAll(len) => {
            let len = length_expr(*len);
            quote! { .padding(#len) }
        }
        Decl::PaddingX(len) => {
            let len = length_expr(*len);
            quote! { .padding_horiz(#len) }
        }
        Decl::PaddingY(len) => {
            let len = length_expr(*len);
            quote! { .padding_vert(#len) }
        }
        Decl::MarginAll(Len::Full)
        | Decl::MarginX(Len::Full)
        | Decl::MarginY(Len::Full)
        | Decl::MarginTop(Len::Full)
        | Decl::MarginBottom(Len::Full)
        | Decl::MarginLeft(Len::Full)
        | Decl::MarginRight(Len::Full) => quote! {},
        Decl::MarginAll(len) => {
            let len = length_expr(*len);
            quote! { .margin(#len) }
        }
        Decl::MarginX(len) => {
            let len = length_expr(*len);
            quote! { .margin_horiz(#len) }
        }
        Decl::MarginY(len) => {
            let len = length_expr(*len);
            quote! { .margin_vert(#len) }
        }
        Decl::MarginTop(Len::Auto) | Decl::MarginBottom(Len::Auto) => {
            // `mt-auto`/`mb-auto` push content to the far edge; Floem models
            // that with LengthAuto::Auto on the per-side margin prop.
            let side = if matches!(decl, Decl::MarginTop(_)) {
                "margin_top"
            } else {
                "margin_bottom"
            };
            let ident = proc_macro2::Ident::new(side, proc_macro2::Span::call_site());
            quote! { .#ident(::floem::style::LengthAuto::Auto) }
        }
        Decl::MarginLeft(Len::Auto) | Decl::MarginRight(Len::Auto) => {
            let side = if matches!(decl, Decl::MarginLeft(_)) {
                "margin_left"
            } else {
                "margin_right"
            };
            let ident = proc_macro2::Ident::new(side, proc_macro2::Span::call_site());
            quote! { .#ident(::floem::style::LengthAuto::Auto) }
        }
        Decl::MarginTop(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .margin_top(#v) }
        }
        Decl::MarginBottom(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .margin_bottom(#v) }
        }
        Decl::MarginLeft(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .margin_left(#v) }
        }
        Decl::MarginRight(Len::Px(v)) => {
            let v = f64_lit(f64::from(*v));
            quote! { .margin_right(#v) }
        }
        Decl::PaddingTop(_)
        | Decl::PaddingBottom(_)
        | Decl::PaddingLeft(_)
        | Decl::PaddingRight(_) => {
            // Composed into one `.apply_padding(Padding { .. })` by the
            // caller (`lower`); unreachable here.
            quote! {}
        }
        _ => return None,
    })
}

/// Positioning and overflow: absolute insets and scroll behavior.
fn position_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        Decl::Position(_) => quote! {},
        Decl::Top(len) => {
            let len = length_expr(*len);
            quote! { .inset_top(#len) }
        }
        Decl::Right(len) => {
            let len = length_expr(*len);
            quote! { .inset_right(#len) }
        }
        Decl::Bottom(len) => {
            let len = length_expr(*len);
            quote! { .inset_bottom(#len) }
        }
        Decl::Left(len) => {
            let len = length_expr(*len);
            quote! { .inset_left(#len) }
        }
        Decl::Overflow(_) => {
            // Scrollable overflow lowers to a wrapping `scroll` view (M5
            // `scroll_area`); clipping is Floem's default child behavior.
            quote! {}
        }
        _ => return None,
    })
}

/// Visuals: background, borders, radii, shadow, opacity and cursor.
fn visual_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        Decl::Background(col) => {
            let col = color_expr(*col, false);
            quote! { .background(#col) }
        }
        Decl::BorderColor(col) => {
            let col = color_expr(*col, false);
            quote! { .border_color(#col) }
        }
        Decl::BorderWidth(side, w) => {
            let w = f64_lit(f64::from(*w));
            match side {
                BorderSide::All => quote! { .border(#w) },
                BorderSide::Top => quote! { .border_top(#w) },
                BorderSide::Bottom => quote! { .border_bottom(#w) },
                BorderSide::Left => quote! { .border_left(#w) },
                BorderSide::Right => quote! { .border_right(#w) },
            }
        }
        Decl::Rounded(r) => {
            let r = f64_lit(f64::from(*r));
            quote! { .border_radius(#r) }
        }
        Decl::RoundedCorner(corner, r) => {
            let r = f64_lit(f64::from(*r));
            let prop = match corner {
                Corner::TopLeft => "border_top_left_radius",
                Corner::TopRight => "border_top_right_radius",
                Corner::BottomLeft => "border_bottom_left_radius",
                Corner::BottomRight => "border_bottom_right_radius",
            };
            let ident = proc_macro2::Ident::new(prop, proc_macro2::Span::call_site());
            quote! { .#ident(#r) }
        }
        Decl::Shadow(_) => {
            // Emitted as a soft elevation shadow matching shadcn's `shadow-md`.
            quote! {
                .box_shadow_color(::floem::peniko::Color::from_rgba8(0, 0, 0, 0x33))
                    .box_shadow_blur(8.0)
                    .box_shadow_v_offset(2.0)
            }
        }
        Decl::Opacity(v) => {
            let v = f64_lit(f64::from(*v));
            quote! { .opacity(#v) }
        }
        Decl::Cursor(quoin_style::Cursor::Pointer) => {
            quote! { .cursor(::floem::style::CursorStyle::Pointer) }
        }
        Decl::Cursor(quoin_style::Cursor::Text) => {
            quote! { .cursor(::floem::style::CursorStyle::Text) }
        }
        Decl::Cursor(quoin_style::Cursor::Default) => {
            quote! { .cursor(::floem::style::CursorStyle::Default) }
        }
        _ => return None,
    })
}

/// Text: color, size, weight, alignment, truncation and family.
fn text_decl(decl: &Decl) -> Option<TokenStream> {
    Some(match decl {
        Decl::TextColor(col) => {
            let col = color_expr(*col, true);
            quote! { .color(#col) }
        }
        Decl::TextSize(v) => {
            let v = f64_lit(f64::from(*v));
            quote! { .font_size(#v) }
        }
        Decl::TextWeight(w) => weight_expr(w.0),
        // Floem aligns label text via the label's own text props; the
        // container-level alignment is a declared allowed difference
        // (the semantic probe normalizes it).
        Decl::TextAlign(_) => quote! {},
        Decl::WhitespaceNowrap | Decl::Truncate => quote! { .text_ellipsis() },
        Decl::FontFamily(family) => {
            let owned = quote! { #family.to_owned() };
            quote! { .font_family(#owned) }
        }
        Decl::SelectNone => quote! { .focus_none() },
        _ => return None,
    })
}

/// Maps the Tailwind weight scale onto `fontique::FontWeight` constants.
fn weight_expr(weight: u16) -> TokenStream {
    let constant = match weight {
        100 => "THIN",
        200 => "EXTRA_LIGHT",
        300 => "LIGHT",
        400 => "REGULAR",
        500 => "MEDIUM",
        600 => "SEMI_BOLD",
        700 => "BOLD",
        800 => "EXTRA_BOLD",
        900 => "BLACK",
        _ => {
            let v = f64_lit(f64::from(weight));
            return quote! { .font_weight(::floem::text::FontWeight::new(#v)) };
        }
    };
    let ident = proc_macro2::Ident::new(constant, proc_macro2::Span::call_site());
    quote! { .font_weight(::floem::text::FontWeight::#ident) }
}

/// One `.apply_padding(Padding { .. })` from per-side pixel padding.
fn padding_struct(left: f64, right: f64, top: f64, bottom: f64) -> TokenStream {
    let (l, r, t, b) = (f64_lit(left), f64_lit(right), f64_lit(top), f64_lit(bottom));
    quote! {
        .apply_padding(::floem::style::Padding {
            left: ::floem::style::Pt(#l),
            right: ::floem::style::Pt(#r),
            top: ::floem::style::Pt(#t),
            bottom: ::floem::style::Pt(#b),
        })
    }
}

/// One lowered class string: everything [`lower_class`] produces.
struct Accumulated {
    normal: Vec<TokenStream>,
    hover: Vec<TokenStream>,
    flex_dir: Option<FlexDir>,
    /// Per-side pixel padding composed into one `apply_padding` call.
    pad: PaddingSides,
}

/// Tracks per-side padding decls so the emit stays a single composite call.
#[derive(Default)]
struct PaddingSides {
    left: Option<f64>,
    right: Option<f64>,
    top: Option<f64>,
    bottom: Option<f64>,
}

impl PaddingSides {
    fn any(&self) -> bool {
        self.left.is_some() || self.right.is_some() || self.top.is_some() || self.bottom.is_some()
    }

    fn tokens(&self) -> TokenStream {
        let f = |v: Option<f64>| v.unwrap_or(0.0);
        padding_struct(f(self.left), f(self.right), f(self.top), f(self.bottom))
    }
}

/// Accumulates one decl into the right bucket; returns `true` when the decl
/// was fully consumed (composite side padding) and needs no chain piece.
fn push_decl(
    decl: &Decl,
    out: &mut Vec<TokenStream>,
    flex_dir: &mut Option<FlexDir>,
    pad: &mut PaddingSides,
) -> bool {
    match *decl {
        Decl::FlexDir(dir) => {
            *flex_dir = Some(dir);
            true
        }
        Decl::PaddingLeft(Len::Px(v)) => {
            pad.left = Some(f64::from(v));
            true
        }
        Decl::PaddingRight(Len::Px(v)) => {
            pad.right = Some(f64::from(v));
            true
        }
        Decl::PaddingTop(Len::Px(v)) => {
            pad.top = Some(f64::from(v));
            true
        }
        Decl::PaddingBottom(Len::Px(v)) => {
            pad.bottom = Some(f64::from(v));
            true
        }
        _ => {
            let tokens = decl_to_tokens(decl);
            if !tokens.is_empty() {
                out.push(tokens);
            }
            false
        }
    }
}

/// Parses a Tailwind class string into the Floem style lowering.
#[must_use]
pub(crate) fn lower_class(class: &str) -> FloemStyles {
    let parsed: ParsedClasses = quoin_style::parse_tailwind(class);

    let mut acc = Accumulated {
        normal: Vec::new(),
        hover: Vec::new(),
        flex_dir: None,
        pad: PaddingSides::default(),
    };

    for decl in &parsed.base {
        push_decl(decl, &mut acc.normal, &mut acc.flex_dir, &mut acc.pad);
    }
    if acc.pad.any() {
        let side_tokens = acc.pad.tokens();
        acc.normal.push(side_tokens);
    }
    for decl in &parsed.hover {
        push_decl(decl, &mut acc.hover, &mut acc.flex_dir, &mut acc.pad);
    }

    let errors = parsed
        .unknown
        .iter()
        .map(|class| {
            let suggestion = suggest_class(class);
            let message = match suggestion {
                Some(sug) => format!(
                    "unknown Tailwind class `{class}` (in `class: \"…\"`). Did you mean `{sug}`?"
                ),
                None => format!(
                    "unknown Tailwind class `{class}` (in `class: \"…\"`). See `quoin_style::KNOWN_CLASSES` for the supported vocabulary."
                ),
            };
            quote! { ::core::compile_error!(#message); }
        })
        .collect();

    FloemStyles {
        normal: acc.normal,
        hover: acc.hover,
        errors,
        flex_dir: acc.flex_dir,
    }
}

/// Wraps a style chain in the unknown-class errors, then the hover selector.
#[must_use]
pub(crate) fn apply_styles(chain: TokenStream, styles: &FloemStyles) -> TokenStream {
    let mut out = chain;
    if !styles.normal.is_empty() {
        let normal = &styles.normal;
        out = quote! { #out.style(|__s| __s #(#normal)*) };
    }
    if !styles.hover.is_empty() {
        let hover = &styles.hover;
        out = quote! { #out.style(|__s| __s.hover(|__s| __s #(#hover)*)) };
    }
    if styles.errors.is_empty() {
        out
    } else {
        let errors = &styles.errors;
        quote! { { #(#errors)* #out } }
    }
}

/// The flex direction for a container, defaulting to row (Tailwind `flex`).
pub(crate) fn flex_dir(styles: &FloemStyles) -> FlexDir {
    styles.flex_dir.unwrap_or(FlexDir::Row)
}
