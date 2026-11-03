//! Tailwind class → typed-`Decl` → GPUI builder-method lowering.
//!
//! The class string is parsed once by [`quoin_style::parse_tailwind`] into
//! typed [`Decl`](quoin_style::Decl) values (the shared style IR), and this
//! module lowers each decl into a chain of GPUI `Styled` trait method calls.
//! Nothing at runtime ever looks at a class string.
//!
//! # Byte-identical output
//!
//! The M1 milestone requires that porting the transpiler onto the style IR
//! does not change GPUI output on existing fixtures. The lowering therefore
//! reproduces the pre-IR token shapes exactly:
//!
//! - palette colors emit hex literals (`.bg(gpui::rgb(0x6b7280))`),
//! - theme tokens and arbitrary colors emit decimal literals
//!   (`.bg(gpui::rgb(1982587))`), exactly as the old tables did,
//! - named font sizes map back onto the named GPUI methods
//!   (`.text_xs()` for `text-xs`),
//! - named weights map onto `FontWeight::THIN`-style constants.
//!
//! # Unknown classes are a compile error
//!
//! Classes outside the vocabulary (see `quoin_style::KNOWN_CLASSES`) produce
//! a `::core::compile_error!` statement with a did-you-mean suggestion,
//! wrapped around the element chain — silent skipping is gone.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use quoin_style::{Col, Decl, Len, ParsedClasses, suggest_class};

/// The GPUI modifier chains produced from one `class=".."` argument.
pub struct TranspiledStyles {
    /// Modifiers applied unconditionally.
    pub normal: Vec<TokenStream>,
    /// Modifiers produced from `hover:`-prefixed classes.
    pub hover: Vec<TokenStream>,
    /// `compile_error!` statements for unrecognized classes.
    pub errors: Vec<TokenStream>,
    /// Whether a scroll overflow decl is present: `overflow_*_scroll` are
    /// `StatefulInteractiveElement` methods in current GPUI, so the div must
    /// be made stateful (`.id(..)`) before the chain applies.
    pub scroll: bool,
}

/// Appends the normal (non-hover) styles to an element chain.
///
/// Unknown-class `compile_error!` statements wrap the chain in a block so
/// the diagnostic fires even though the chain itself is syntactically
/// unaffected.
#[must_use]
pub fn apply_normal(chain: TokenStream, styles: &TranspiledStyles) -> TokenStream {
    let mut out = chain;
    for style in &styles.normal {
        out = quote! { #out #style };
    }
    wrap_errors(out, styles)
}

/// Wraps a chain in unknown-class `compile_error!` statements, if any.
#[must_use]
pub fn wrap_errors(chain: TokenStream, styles: &TranspiledStyles) -> TokenStream {
    if styles.errors.is_empty() {
        chain
    } else {
        let errors = &styles.errors;
        quote! { { #(#errors)* #chain } }
    }
}

/// Appends the normal styles and then the hover styles
/// (`.hover(|__s| __s …)`) to an element chain.
#[must_use]
pub fn apply_normal_and_hover(chain: TokenStream, styles: &TranspiledStyles) -> TokenStream {
    let mut out = chain;
    for style in &styles.normal {
        out = quote! { #out #style };
    }
    if !styles.hover.is_empty() {
        let hover_tokens = &styles.hover;
        out = quote! { #out.hover(|__s| __s #(#hover_tokens)*) };
    }
    wrap_errors(out, styles)
}

/// Transpiles a Tailwind `class=".."` string into GPUI modifier chains,
/// splitting `hover:`-prefixed classes into their own list and collecting
/// unknown classes into `compile_error!` statements.
#[must_use]
pub fn transpile_class(class_str: &str) -> TranspiledStyles {
    let parsed = quoin_style::parse_tailwind(class_str);
    let normal = parsed.base.iter().map(decl_to_gpui_tokens).collect();
    let hover = parsed.hover.iter().map(decl_to_gpui_tokens).collect();
    let errors = unknown_class_errors(&parsed);
    let scroll = parsed
        .base
        .iter()
        .chain(&parsed.hover)
        .any(|decl| matches!(decl, Decl::Overflow(o) if o.needs_stateful_scroll()));
    TranspiledStyles {
        normal,
        hover,
        errors,
        scroll,
    }
}

/// Builds one `compile_error!` per unknown class, with a did-you-mean
/// suggestion when the edit distance finds a close neighbor.
#[must_use]
pub fn unknown_class_errors(parsed: &ParsedClasses) -> Vec<TokenStream> {
    parsed
        .unknown
        .iter()
        .map(|class| {
            let suggestion = suggest_class(class);
            let message = match suggestion {
                Some(sug) => {
                    format!("unknown Tailwind class `{class}` (in `class: \"…\"`). Did you mean `{sug}`?")
                }
                None => format!(
                    "unknown Tailwind class `{class}` (in `class: \"…\"`). See `quoin_style::KNOWN_CLASSES` for the supported vocabulary."
                ),
            };
            quote! { ::core::compile_error!(#message); }
        })
        .collect()
}

/// Formats an `f32` the way the pre-IR transpiler's literals were written:
/// `4.0`, `0.25`, `9999.0` (the `Debug` formatting of `f32`).
fn f32_lit(value: f32) -> TokenStream {
    let text = format!("{value:?}");
    let lit: proc_macro2::Literal =
        syn::parse_str(&text).unwrap_or_else(|_| proc_macro2::Literal::f32_suffixed(value));
    lit.to_token_stream()
}

/// The GPUI token for one pixel literal: `gpui::px(4.0)`.
fn px(value: f32) -> TokenStream {
    let lit = f32_lit(value);
    quote! { gpui::px(#lit) }
}

/// Emits a decimal `u32` (the pre-IR token-table form for theme tokens and
/// arbitrary colors).
fn decimal(value: u32) -> TokenStream {
    let lit = proc_macro2::Literal::u32_unsuffixed(value);
    lit.to_token_stream()
}

/// Emits a hex `u32` literal using the source text (`0x6b7280`), preserving
/// the exact spelling of the pre-IR palette table.
fn hex_literal(hex: quoin_style::HexLit) -> TokenStream {
    let text = if hex.text.is_empty() {
        // Arbitrary colors have no static text; the old transpiler emitted
        // these as decimals anyway.
        return decimal(hex.value);
    } else {
        format!("0x{}", hex.text)
    };
    let lit: proc_macro2::Literal =
        syn::parse_str(&text).unwrap_or_else(|_| proc_macro2::Literal::u32_unsuffixed(hex.value));
    lit.to_token_stream()
}

/// The `rgb`/`rgba` constructor for a packed `0xRRGGBBAA` value, matching
/// the pre-IR choice: opaque colors go through `rgb` (6-digit decimal),
/// translucent ones through `rgba`.
fn color_expr(hex: quoin_style::HexLit) -> TokenStream {
    if hex.alpha() == 0xff {
        let rgb = hex.value >> 8;
        let lit = decimal(rgb);
        quote! { gpui::rgb(#lit) }
    } else {
        let lit = decimal(hex.value);
        quote! { gpui::rgba(#lit) }
    }
}

/// The GPUI color for a [`Col`] in the background/border role.
fn bg_color(col: &Col) -> TokenStream {
    match col {
        Col::Rgb(hex) => color_expr_preserving_palette(*hex),
        Col::Transparent => quote! { gpui::transparent_black() },
        Col::White => quote! { gpui::white() },
        Col::Black => quote! { gpui::black() },
        Col::Token(token) => {
            let surface = token.surface_hex();
            let lit = decimal(surface.value >> 8);
            quote! { gpui::rgb(#lit) }
        }
    }
}

/// The GPUI color for a [`Col`] in the text role.
fn text_color(col: &Col) -> TokenStream {
    match col {
        Col::Rgb(hex) => color_expr_preserving_palette(*hex),
        Col::Transparent => quote! { gpui::transparent_black() },
        Col::White => quote! { gpui::white() },
        Col::Black => quote! { gpui::black() },
        Col::Token(token) => {
            let text = token.text_hex();
            let lit = decimal(text.value >> 8);
            quote! { gpui::rgb(#lit) }
        }
    }
}

/// Palette colors (which carry their source text) emit hex literals like the
/// pre-IR tables; arbitrary colors emit the packed decimal the old
/// arbitrary-color path produced.
fn color_expr_preserving_palette(hex: quoin_style::HexLit) -> TokenStream {
    if hex.text.is_empty() {
        color_expr(hex)
    } else {
        let lit = hex_literal(hex);
        quote! { gpui::rgb(#lit) }
    }
}

/// Zero-valued spacing classes map onto the dedicated GPUI `_0` methods
/// (`.gap_0()`, `.p_0()`, …), exactly like the pre-IR table.
fn spacing_call(zero_method: &str, value: f32, method: &str) -> TokenStream {
    if value == 0.0 {
        let ident = proc_macro2::Ident::new(zero_method, proc_macro2::Span::call_site());
        quote! { .#ident() }
    } else {
        let ident = proc_macro2::Ident::new(method, proc_macro2::Span::call_site());
        let lit = px(value);
        quote! { .#ident(#lit) }
    }
}

/// Emits `text-size` for a value that is not one of the named scale steps.
fn text_size_tokens(value: f32) -> TokenStream {
    if let Some(scale) = quoin_style::decl::TextScale::from_px(value) {
        let ident = match scale {
            quoin_style::decl::TextScale::Xs => "text_xs",
            quoin_style::decl::TextScale::Sm => "text_sm",
            quoin_style::decl::TextScale::Base => "text_base",
            quoin_style::decl::TextScale::Lg => "text_lg",
            quoin_style::decl::TextScale::Xl => "text_xl",
            quoin_style::decl::TextScale::Xxl => "text_2xl",
            quoin_style::decl::TextScale::Xxl3 => "text_3xl",
            quoin_style::decl::TextScale::Xxl4 => "text_4xl",
        };
        let ident = proc_macro2::Ident::new(ident, proc_macro2::Span::call_site());
        quote! { .#ident() }
    } else {
        let lit = px(value);
        quote! { .text_size(#lit) }
    }
}

/// Lowers one [`Decl`] into a GPUI builder-method chain segment.
// A flat class→builder lookup table: splitting it would scatter the
// vocabulary across helpers and hurt auditability against `KNOWN_CLASSES`.
#[expect(clippy::too_many_lines)]
#[must_use]
pub fn decl_to_gpui_tokens(decl: &Decl) -> TokenStream {
    match decl {
        // ── Layout ──────────────────────────────────────────────────────
        Decl::Display(d) => match d {
            quoin_style::decl::Display::Flex => quote! { .flex() },
            quoin_style::decl::Display::InlineFlex => quote! { .inline_flex() },
            quoin_style::decl::Display::Block => quote! { .block() },
            quoin_style::decl::Display::InlineBlock => quote! { .inline_block() },
            quoin_style::decl::Display::Hidden => quote! { .hidden() },
        },
        Decl::FlexDir(dir) => match dir {
            quoin_style::decl::FlexDir::Row => quote! { .flex_row() },
            quoin_style::decl::FlexDir::Column => quote! { .flex_col() },
            quoin_style::decl::FlexDir::RowReverse => quote! { .flex_row_reverse() },
            quoin_style::decl::FlexDir::ColumnReverse => quote! { .flex_col_reverse() },
        },
        Decl::FlexWrap(true) => quote! { .flex_wrap() },
        Decl::FlexWrap(false) => quote! { .flex_nowrap() },
        Decl::Grow(v) => {
            let lit = f32_lit(*v);
            quote! { .flex_grow(#lit) }
        }
        Decl::Shrink(v) => {
            let lit = f32_lit(*v);
            quote! { .flex_shrink(#lit) }
        }
        Decl::Flex1 => quote! { .flex_1() },
        Decl::Justify(j) => match j {
            quoin_style::decl::Justify::Start => quote! { .justify_start() },
            quoin_style::decl::Justify::End => quote! { .justify_end() },
            quoin_style::decl::Justify::Center => quote! { .justify_center() },
            quoin_style::decl::Justify::Between => quote! { .justify_between() },
            quoin_style::decl::Justify::Around => quote! { .justify_around() },
            quoin_style::decl::Justify::Evenly => quote! { .justify_evenly() },
        },
        Decl::Items(i) => match i {
            quoin_style::decl::Items::Start => quote! { .items_start() },
            quoin_style::decl::Items::End => quote! { .items_end() },
            quoin_style::decl::Items::Center => quote! { .items_center() },
            quoin_style::decl::Items::Baseline => quote! { .items_baseline() },
            quoin_style::decl::Items::Stretch => quote! { .items_stretch() },
        },
        Decl::SelfAlign(i) => match i {
            quoin_style::decl::Items::Start => quote! { .self_start() },
            quoin_style::decl::Items::End => quote! { .self_end() },
            quoin_style::decl::Items::Center => quote! { .self_center() },
            quoin_style::decl::Items::Baseline | quoin_style::decl::Items::Stretch => {
                quote! { .self_center() }
            }
        },
        Decl::Gap(Len::Px(v)) => spacing_call("gap_0", *v, "gap"),
        Decl::Gap(_) => quote! { .gap_0() },

        // ── Sizing ──────────────────────────────────────────────────────
        Decl::Width(Len::Full) => quote! { .w_full() },
        Decl::Width(Len::Auto) => quote! { .w_auto() },
        Decl::Width(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .w(#lit) }
        }
        Decl::Height(Len::Full) => quote! { .h_full() },
        Decl::Height(Len::Auto) => quote! { .h_auto() },
        Decl::Height(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .h(#lit) }
        }
        Decl::SizeFull => quote! { .size_full() },
        Decl::MinWidth(Len::Px(0.0)) => quote! { .min_w_0() },
        Decl::MinWidth(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .min_w(#lit) }
        }
        Decl::MinWidth(_) => quote! { .min_w_0() },
        Decl::MinHeight(Len::Px(0.0)) => quote! { .min_h_0() },
        Decl::MinHeight(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .min_h(#lit) }
        }
        Decl::MinHeight(_) => quote! { .min_h_0() },
        Decl::MaxWidth(Len::Full | Len::Auto) => quote! { .max_w_full() },
        Decl::MaxWidth(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .max_w(#lit) }
        }
        Decl::MaxHeight(Len::Full | Len::Auto) => quote! { .max_h_full() },
        Decl::MaxHeight(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .max_h(#lit) }
        }

        // ── Spacing ─────────────────────────────────────────────────────
        Decl::PaddingAll(Len::Px(v)) => spacing_call("p_0", *v, "p"),
        Decl::PaddingAll(_) => quote! { .p_0() },
        Decl::PaddingX(Len::Px(v)) => spacing_call("px_0", *v, "px"),
        Decl::PaddingX(_) => quote! { .px_0() },
        Decl::PaddingY(Len::Px(v)) => spacing_call("py_0", *v, "py"),
        Decl::PaddingY(_) => quote! { .py_0() },
        Decl::PaddingTop(Len::Px(v)) => spacing_call("pt_0", *v, "pt"),
        Decl::PaddingTop(_) => quote! { .pt_0() },
        Decl::PaddingBottom(Len::Px(v)) => spacing_call("pb_0", *v, "pb"),
        Decl::PaddingBottom(_) => quote! { .pb_0() },
        Decl::PaddingLeft(Len::Px(v)) => spacing_call("pl_0", *v, "pl"),
        Decl::PaddingLeft(_) => quote! { .pl_0() },
        Decl::PaddingRight(Len::Px(v)) => spacing_call("pr_0", *v, "pr"),
        Decl::PaddingRight(_) => quote! { .pr_0() },
        Decl::MarginAll(Len::Px(v)) => spacing_call("m_0", *v, "m"),
        Decl::MarginAll(_) => quote! { .m_0() },
        Decl::MarginX(Len::Auto | Len::Full) => quote! { .mx_auto() },
        Decl::MarginX(Len::Px(v)) => spacing_call("mx_0", *v, "mx"),
        Decl::MarginY(Len::Px(v)) => spacing_call("my_0", *v, "my"),
        Decl::MarginY(_) => quote! { .my_0() },
        Decl::MarginTop(Len::Px(v)) => spacing_call("mt_0", *v, "mt"),
        Decl::MarginTop(_) => quote! { .mt_0() },
        Decl::MarginBottom(Len::Px(v)) => spacing_call("mb_0", *v, "mb"),
        Decl::MarginBottom(_) => quote! { .mb_0() },
        Decl::MarginLeft(Len::Auto | Len::Full) => quote! { .ml_auto() },
        Decl::MarginLeft(Len::Px(v)) => spacing_call("ml_0", *v, "ml"),
        Decl::MarginRight(Len::Auto | Len::Full) => quote! { .mr_auto() },
        Decl::MarginRight(Len::Px(v)) => spacing_call("mr_0", *v, "mr"),

        // ── Position & overflow ─────────────────────────────────────────
        Decl::Position(quoin_style::decl::Position::Absolute) => quote! { .absolute() },
        Decl::Position(quoin_style::decl::Position::Relative) => quote! { .relative() },
        Decl::Top(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .top(#lit) }
        }
        Decl::Top(_) => {
            let lit = px(0.0);
            quote! { .top(#lit) }
        }
        Decl::Right(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .right(#lit) }
        }
        Decl::Right(_) => {
            let lit = px(0.0);
            quote! { .right(#lit) }
        }
        Decl::Bottom(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .bottom(#lit) }
        }
        Decl::Bottom(_) => {
            let lit = px(0.0);
            quote! { .bottom(#lit) }
        }
        Decl::Left(Len::Px(v)) => {
            let lit = px(*v);
            quote! { .left(#lit) }
        }
        Decl::Left(_) => {
            let lit = px(0.0);
            quote! { .left(#lit) }
        }
        Decl::Overflow(o) => match o {
            quoin_style::decl::Overflow::Hidden => quote! { .overflow_hidden() },
            quoin_style::decl::Overflow::ScrollY => quote! { .overflow_y_scroll() },
            quoin_style::decl::Overflow::ScrollX | quoin_style::decl::Overflow::AutoX => {
                quote! { .overflow_x_scroll() }
            }
            quoin_style::decl::Overflow::Scroll => quote! { .overflow_scroll() },
            quoin_style::decl::Overflow::AutoY => quote! { .overflow_auto() },
        },

        // ── Visual ──────────────────────────────────────────────────────
        Decl::Background(col) => {
            let color = bg_color(col);
            quote! { .bg(#color) }
        }
        Decl::BorderColor(col) => {
            let color = text_color(col);
            quote! { .border_color(#color) }
        }
        Decl::BorderWidth(side, width) => {
            // The pre-IR vocabulary only had 0/1/2-px borders; other widths
            // snap to the nearest supported thickness.
            let nearest = if *width == 0.0 {
                0.0
            } else if *width >= 1.5 {
                2.0
            } else {
                1.0
            };
            match side {
                quoin_style::decl::BorderSide::All => match nearest {
                    0.0 => quote! { .border_0() },
                    2.0 => quote! { .border_2() },
                    _ => quote! { .border_1() },
                },
                quoin_style::decl::BorderSide::Bottom => match nearest {
                    0.0 => quote! { .border_b_0() },
                    2.0 => quote! { .border_b_2() },
                    _ => quote! { .border_b_1() },
                },
                quoin_style::decl::BorderSide::Top => match nearest {
                    0.0 => quote! { .border_t_0() },
                    2.0 => quote! { .border_t_2() },
                    _ => quote! { .border_t_1() },
                },
                quoin_style::decl::BorderSide::Left => {
                    if nearest == 0.0 {
                        quote! { .border_l_0() }
                    } else {
                        quote! { .border_l_1() }
                    }
                }
                quoin_style::decl::BorderSide::Right => {
                    if nearest == 0.0 {
                        quote! { .border_r_0() }
                    } else {
                        quote! { .border_r_1() }
                    }
                }
            }
        }
        Decl::Rounded(v) => {
            let lit = px(*v);
            quote! { .rounded(#lit) }
        }
        Decl::RoundedCorner(corner, v) => {
            let lit = px(*v);
            let method = match corner {
                quoin_style::decl::Corner::TopLeft => "rounded_tl",
                quoin_style::decl::Corner::TopRight => "rounded_tr",
                quoin_style::decl::Corner::BottomLeft => "rounded_bl",
                quoin_style::decl::Corner::BottomRight => "rounded_br",
            };
            let ident = proc_macro2::Ident::new(method, proc_macro2::Span::call_site());
            quote! { .#ident(#lit) }
        }
        Decl::Shadow(s) => match s {
            quoin_style::decl::Shadow::Xs => quote! { .shadow_xs() },
            quoin_style::decl::Shadow::Sm => quote! { .shadow_sm() },
            quoin_style::decl::Shadow::Md => quote! { .shadow_md() },
            quoin_style::decl::Shadow::Lg => quote! { .shadow_lg() },
            quoin_style::decl::Shadow::Xl => quote! { .shadow_xl() },
        },
        Decl::Opacity(v) => {
            let lit = f32_lit(*v);
            quote! { .opacity(#lit) }
        }
        Decl::Cursor(c) => match c {
            quoin_style::decl::Cursor::Pointer => quote! { .cursor_pointer() },
            quoin_style::decl::Cursor::Default => quote! { .cursor_default() },
            quoin_style::decl::Cursor::Text => quote! { .cursor_text() },
        },

        // ── Text ────────────────────────────────────────────────────────
        Decl::TextColor(col) => {
            let color = text_color(col);
            quote! { .text_color(#color) }
        }
        Decl::TextSize(v) => text_size_tokens(*v),
        Decl::TextWeight(w) => match w.0 {
            100 => quote! { .font_weight(gpui::FontWeight::THIN) },
            300 => quote! { .font_weight(gpui::FontWeight::LIGHT) },
            400 => quote! { .font_weight(gpui::FontWeight::NORMAL) },
            500 => quote! { .font_weight(gpui::FontWeight::MEDIUM) },
            600 => quote! { .font_weight(gpui::FontWeight::SEMIBOLD) },
            700 => quote! { .font_weight(gpui::FontWeight::BOLD) },
            other => {
                let lit = f32_lit(f32::from(other));
                quote! { .font_weight(gpui::FontWeight::new(#lit)) }
            }
        },
        Decl::TextAlign(a) => match a {
            quoin_style::decl::TextAlign::Center => quote! { .text_center() },
            quoin_style::decl::TextAlign::Left => quote! { .text_left() },
            quoin_style::decl::TextAlign::Right => quote! { .text_right() },
        },
        Decl::WhitespaceNowrap => quote! { .whitespace_nowrap() },
        Decl::Truncate => quote! { .truncate() },
        Decl::FontFamily(family) => quote! { .font_family(#family) },
        Decl::SelectNone => quote! { .select_none() },
    }
}
