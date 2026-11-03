//! `Decl` → Xilem style lowering.
//!
//! Xilem styles are per-widget properties (`xilem_masonry::style::Style`
//! trait): `.width(Dim)`, `.padding(Padding)`, `.gap(Length)`,
//! `.background(Color)`, `.border(Color, Length)`, `.corner_radius(Length)`.
//! Text decls never land here — they cascade through
//! [`super::TextStyleCx`](super::TextStyleCx) onto the labels.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use quoin_style::{Col, Decl, FlexDir, Items, Justify, Len, ParsedClasses, suggest_class};

/// The box-model style pieces a container needs.
pub(crate) struct XilemStyles {
    /// Flex-level decls (gap / main / cross alignment).
    pub flex: Vec<TokenStream>,
    /// Box-model decls applied through a `sized_box` wrap (or directly on
    /// the flex when the widget carries the property).
    pub box_chain: Vec<TokenStream>,
    /// `compile_error!` statements for unrecognized classes.
    pub errors: Vec<TokenStream>,
    /// The flex direction declared by `flex-col`/`flex-row`-family classes.
    pub flex_dir: Option<FlexDir>,
}

/// The `::xilem::Color` expression for a style-IR color. Text-role colors
/// read the theme's text table (`<token>_text()`), matching the GPUI
/// backend's `text-*` approximations.
pub(crate) fn color_expr(col: Col, text_role: bool) -> TokenStream {
    match col {
        Col::Rgb(lit) => {
            let (r, g, b, a) = (lit.red(), lit.green(), lit.blue(), lit.alpha());
            quote! { ::xilem::Color::from_rgba8(#r, #g, #b, #a) }
        }
        Col::Transparent => quote! { ::xilem::Color::TRANSPARENT },
        Col::White => quote! { ::xilem::Color::WHITE },
        Col::Black => quote! { ::xilem::Color::BLACK },
        Col::Token(token) => {
            let suffix = if text_role { "_text" } else { "" };
            let fn_name = format!("{}{suffix}", token.fn_name());
            let ident = proc_macro2::Ident::new(&fn_name, proc_macro2::Span::call_site());
            quote! { ::quoin::xilem_theme::#ident() }
        }
    }
}

/// The `masonry::layout::Length` expression for a length.
pub(crate) fn length_expr(len: Len) -> TokenStream {
    match len {
        Len::Px(v) => {
            let v = f64::from(v);
            quote! { ::xilem::masonry::layout::Length::px(#v) }
        }
        Len::Full => quote! { ::xilem::masonry::layout::Length::px(f64::MAX) },
        Len::Auto => quote! { ::xilem::masonry::layout::Length::ZERO },
    }
}

/// The `FontWeight` constant for a Tailwind weight.
pub(crate) fn weight_const(weight: u16) -> TokenStream {
    let constant = match weight {
        100 => "THIN",
        200 => "EXTRA_LIGHT",
        300 => "LIGHT",
        400 => "NORMAL",
        500 => "MEDIUM",
        600 => "SEMI_BOLD",
        700 => "BOLD",
        800 => "EXTRA_BOLD",
        900 => "BLACK",
        _ => {
            let v = f32::from(weight);
            return quote! { ::xilem::FontWeight::new(#v) };
        }
    };
    let ident = proc_macro2::Ident::new(constant, proc_macro2::Span::call_site());
    quote! { ::xilem::FontWeight::#ident }
}

/// A float literal formatted like Rust's `f32` debug output.
pub(crate) fn f32_lit(value: f32) -> TokenStream {
    let text = format!("{value:?}");
    let lit: proc_macro2::Literal =
        syn::parse_str(&text).unwrap_or_else(|_| proc_macro2::Literal::f32_suffixed(value));
    lit.to_token_stream()
}

/// Lowers one non-text decl into its bucket. Returns whether the decl was
/// consumed (composite side padding accumulates into `pad_*`).
// Full/auto paddings and margins are intentionally no-ops with identical
// bodies across decls; the patterns (not the bodies) carry the semantics.
#[expect(clippy::too_many_lines, clippy::match_same_arms)]
pub(crate) fn push_decl(
    decl: &Decl,
    flex: &mut Vec<TokenStream>,
    box_chain: &mut Vec<TokenStream>,
    flex_dir: &mut Option<FlexDir>,
    pad: &mut [Option<f64>; 4],
) -> bool {
    match *decl {
        Decl::FlexDir(dir) => {
            *flex_dir = Some(dir);
            true
        }
        Decl::Gap(len) => {
            let len = length_expr(len);
            flex.push(quote! { .gap(#len) });
            true
        }
        Decl::Justify(justify) => {
            let variant = match justify {
                Justify::Start => "Start",
                Justify::End => "End",
                Justify::Center => "Center",
                Justify::Between => "SpaceBetween",
                Justify::Around => "SpaceAround",
                Justify::Evenly => "SpaceEvenly",
            };
            let ident = proc_macro2::Ident::new(variant, proc_macro2::Span::call_site());
            flex.push(quote! {
                .main_axis_alignment(::xilem::view::MainAxisAlignment::#ident)
            });
            true
        }
        Decl::Items(items) => {
            let variant = match items {
                Items::Start => "Start",
                Items::End => "End",
                Items::Center => "Center",
                Items::Baseline => "Baseline",
                Items::Stretch => "Stretch",
            };
            let ident = proc_macro2::Ident::new(variant, proc_macro2::Span::call_site());
            flex.push(quote! {
                .cross_axis_alignment(::xilem::view::CrossAxisAlignment::#ident)
            });
            true
        }
        Decl::PaddingAll(Len::Px(v)) => {
            let v = f64::from(v);
            pad[0] = Some(v);
            pad[1] = Some(v);
            pad[2] = Some(v);
            pad[3] = Some(v);
            true
        }
        Decl::PaddingAll(_) => true,
        Decl::PaddingTop(Len::Full | Len::Auto) => true,
        Decl::PaddingTop(Len::Px(v)) => {
            pad[0] = Some(f64::from(v));
            true
        }
        Decl::PaddingBottom(Len::Full | Len::Auto) => true,
        Decl::PaddingBottom(Len::Px(v)) => {
            pad[1] = Some(f64::from(v));
            true
        }
        Decl::PaddingLeft(Len::Full | Len::Auto) => true,
        Decl::PaddingLeft(Len::Px(v)) => {
            pad[2] = Some(f64::from(v));
            true
        }
        Decl::PaddingRight(Len::Full | Len::Auto) => true,
        Decl::PaddingRight(Len::Px(v)) => {
            pad[3] = Some(f64::from(v));
            true
        }
        Decl::PaddingX(Len::Full | Len::Auto) => true,
        Decl::PaddingX(Len::Px(v)) => {
            pad[2] = Some(f64::from(v));
            pad[3] = Some(f64::from(v));
            true
        }
        Decl::PaddingY(Len::Full | Len::Auto) => true,
        Decl::PaddingY(Len::Px(v)) => {
            pad[0] = Some(f64::from(v));
            pad[1] = Some(f64::from(v));
            true
        }
        Decl::Width(len) => {
            let dim = dim_expr(len);
            box_chain.push(quote! { .width(#dim) });
            true
        }
        Decl::Height(len) => {
            let dim = dim_expr(len);
            box_chain.push(quote! { .height(#dim) });
            true
        }
        Decl::SizeFull => {
            box_chain.push(quote! {
                .width(::xilem::masonry::layout::Dim::Stretch)
                    .height(::xilem::masonry::layout::Dim::Stretch)
            });
            true
        }
        Decl::MinWidth(_) | Decl::MaxWidth(_) | Decl::MinHeight(_) | Decl::MaxHeight(_) => {
            // Masonry has no per-side min/max dimension properties; the
            // sized_box carries the fixed ones and min/max are a declared
            // allowed difference on Xilem.
            true
        }
        Decl::Background(col) => {
            let col = color_expr(col, false);
            box_chain.push(quote! { .background(#col) });
            true
        }
        Decl::BorderColor(col) => {
            let col = color_expr(col, false);
            box_chain.push(quote! { .border_color(#col) });
            true
        }
        Decl::BorderWidth(quoin_style::BorderSide::All, w) => {
            let len = length_expr(Len::Px(w));
            box_chain.push(quote! { .border_width(#len) });
            true
        }
        Decl::BorderWidth(..) => true, // per-side borders: allowed difference
        Decl::Rounded(r) => {
            let len = length_expr(Len::Px(r));
            box_chain.push(quote! { .corner_radius(#len) });
            true
        }
        Decl::RoundedCorner(..) => true, // per-corner radius: allowed difference
        Decl::Opacity(v) => {
            // Masonry has no opacity property; a declared allowed difference.
            let _ = v;
            true
        }
        Decl::Position(_) | Decl::Top(_) | Decl::Right(_) | Decl::Bottom(_) | Decl::Left(_) => {
            // Absolute positioning is not part of the v1 Xilem path.
            true
        }
        Decl::Overflow(_) => true, // scroll wraps through `scroll_area`
        Decl::MarginAll(Len::Px(v)) => {
            // Margins map onto padding of the parent-positioned element;
            // Xilem models them per flex item (`.flex(..)`), which the
            // emitter applies for direct flex children.
            let v = f64::from(v);
            box_chain.push(quote! { .flex(#v) });
            true
        }
        Decl::MarginAll(_) | Decl::MarginX(_) | Decl::MarginY(_) => true,
        Decl::MarginTop(_) | Decl::MarginBottom(_) | Decl::MarginLeft(_) | Decl::MarginRight(_) => {
            true
        }
        Decl::Grow(v) => {
            let v = f32_lit(v);
            box_chain.push(quote! { .flex(#v) });
            true
        }
        Decl::Shrink(_) | Decl::Flex1 => {
            box_chain.push(quote! { .flex(1.0) });
            true
        }
        Decl::FlexWrap(_) => true, // masonry flex has no wrap
        Decl::SelfAlign(_) => true,
        Decl::Cursor(_) => true, // no cursor property on masonry widgets
        Decl::Shadow(_) => true, // box shadows are a declared difference
        // Text decls cascade through TextStyleCx; never emitted here.
        Decl::TextColor(_)
        | Decl::TextSize(_)
        | Decl::TextWeight(_)
        | Decl::TextAlign(_)
        | Decl::WhitespaceNowrap
        | Decl::Truncate
        | Decl::FontFamily(_)
        | Decl::SelectNone => true,
        Decl::Display(_) => true, // Xilem is always flex
    }
}

/// The `Dim` expression for a length (`Stretch` for `w-full`).
fn dim_expr(len: Len) -> TokenStream {
    match len {
        Len::Full => quote! { ::xilem::masonry::layout::Dim::Stretch },
        Len::Px(v) => {
            let len = length_expr(Len::Px(v));
            quote! { ::xilem::masonry::layout::Dim::Fixed(#len) }
        }
        Len::Auto => quote! { ::xilem::masonry::layout::Dim::Auto },
    }
}

/// Composes the accumulated per-side padding into one `Padding` value.
pub(crate) fn padding_expr(pad: &[Option<f64>; 4]) -> Option<TokenStream> {
    if !pad.iter().any(Option::is_some) {
        return None;
    }
    let top = pad[0].unwrap_or(0.0);
    let bottom = pad[1].unwrap_or(0.0);
    let left = pad[2].unwrap_or(0.0);
    let right = pad[3].unwrap_or(0.0);
    Some(quote! {
        .padding(::xilem::masonry::properties::Padding {
            top: ::xilem::masonry::layout::Length::px(#top),
            bottom: ::xilem::masonry::layout::Length::px(#bottom),
            left: ::xilem::masonry::layout::Length::px(#left),
            right: ::xilem::masonry::layout::Length::px(#right),
        })
    })
}

/// Parses a Tailwind class string into the Xilem style lowering.
#[must_use]
pub(crate) fn lower_class(class: &str, text: &mut super::TextStyleCx) -> XilemStyles {
    let parsed: ParsedClasses = quoin_style::parse_tailwind(class);

    let mut flex = Vec::new();
    let mut box_chain = Vec::new();
    let mut flex_dir = None;
    let mut pad = [None; 4];

    for decl in &parsed.base {
        match decl {
            Decl::TextColor(col) => text.color = Some(color_expr(*col, true)),
            Decl::TextSize(v) => text.size = Some(*v),
            Decl::TextWeight(w) => text.weight = Some(w.0),
            Decl::TextAlign(a) => {
                let variant = match a {
                    quoin_style::TextAlign::Left => "Start",
                    quoin_style::TextAlign::Center => "Center",
                    quoin_style::TextAlign::Right => "End",
                };
                let ident = proc_macro2::Ident::new(variant, proc_macro2::Span::call_site());
                text.alignment = Some(quote! { ::xilem::TextAlign::#ident });
            }
            _ => {
                push_decl(decl, &mut flex, &mut box_chain, &mut flex_dir, &mut pad);
            }
        }
    }
    if let Some(pad_tokens) = padding_expr(&pad) {
        box_chain.push(pad_tokens);
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

    XilemStyles {
        flex,
        box_chain,
        errors,
        flex_dir,
    }
}

/// Wraps a chain in the unknown-class errors.
#[must_use]
pub(crate) fn wrap_errors(chain: TokenStream, styles: &XilemStyles) -> TokenStream {
    if styles.errors.is_empty() {
        chain
    } else {
        let errors = &styles.errors;
        quote! { { #(#errors)* #chain } }
    }
}

/// The flex direction for a container, defaulting to row.
pub(crate) fn flex_dir(styles: &XilemStyles) -> FlexDir {
    styles.flex_dir.unwrap_or(FlexDir::Row)
}
