//! Tailwind CSS → GPUI builder-method transpiler.
//!
//! Converts Tailwind class strings into chains of GPUI `Styled` trait methods.
//! This is the primary mechanism by which `quoin_render!` applies visual styling
//! in GPUI output.
//!
//! # How It Works
//!
//! 1. [`transpile_class`] splits the input string on whitespace.
//! 2. Classes prefixed with `hover:` are separated into a hover list.
//! 3. Each class is looked up in [`transpile_single_class`], which returns a
//!    `TokenStream` representing the corresponding GPUI method call.
//! 4. Normal classes are collected into `TranspiledStyles::normal`; hover
//!    classes into `TranspiledStyles::hover`.
//! 5. The caller (in `render_gpui.rs`) applies normal styles directly via
//!    method chaining, and hover styles via `.hover(|s| s …)`.
//!
//! # Coverage
//!
//! Currently supports: display, flexbox, alignment, gap, padding, margin,
//! width/height, opacity, background colors, text colors, font sizes,
//! font weights, border radius, cursor, position, overflow, whitespace,
//! text alignment, borders, and border colors.
//!
//! Classes that are not recognized return `None` from `transpile_single_class`
//! and are silently skipped. For Leptos and Dioxus, Tailwind classes are
//! passed through as raw `class=` attribute strings (no transpilation needed).

use proc_macro2::TokenStream;
use quote::quote;

/// The GPUI modifier chains produced from one `class=".."` argument.
pub struct TranspiledStyles {
    /// Modifiers applied unconditionally.
    pub normal: Vec<TokenStream>,
    /// Modifiers produced from `hover:`-prefixed classes.
    pub hover: Vec<TokenStream>,
}

/// Transpiles a Tailwind `class=".."` string into GPUI modifier chains,
/// splitting `hover:`-prefixed classes into their own list.
#[must_use]
pub fn transpile_class(class_str: &str) -> TranspiledStyles {
    let mut normal = Vec::new();
    let mut hover = Vec::new();

    for class in class_str.split_whitespace() {
        if let Some(rest) = class.strip_prefix("hover:") {
            if let Some(token) = transpile_single_class(rest) {
                hover.push(token);
            }
            continue;
        }

        if let Some(token) = transpile_single_class(class) {
            normal.push(token);
        }
    }

    TranspiledStyles { normal, hover }
}

fn transpile_layout_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Display
        "flex" => quote! { .flex() },
        "inline-flex" => quote! { .inline_flex() },
        "block" => quote! { .block() },
        "inline-block" => quote! { .inline_block() },
        "hidden" => quote! { .hidden() },
        // Flex direction
        "flex-col" => quote! { .flex_col() },
        "flex-row" => quote! { .flex_row() },
        "flex-col-reverse" => quote! { .flex_col_reverse() },
        "flex-row-reverse" => quote! { .flex_row_reverse() },
        // Flex wrap
        "flex-wrap" => quote! { .flex_wrap() },
        "flex-nowrap" => quote! { .flex_nowrap() },
        // Flex grow/shrink
        "flex-grow" | "grow" => quote! { .flex_grow(1.0) },
        "flex-grow-0" | "grow-0" => quote! { .flex_grow(0.0) },
        "flex-shrink" | "shrink" => quote! { .flex_shrink(1.0) },
        "flex-shrink-0" | "shrink-0" => quote! { .flex_shrink(0.0) },
        // Alignment
        "items-start" => quote! { .items_start() },
        "items-end" => quote! { .items_end() },
        "items-center" => quote! { .items_center() },
        "items-baseline" => quote! { .items_baseline() },
        "items-stretch" => quote! { .items_stretch() },
        "justify-start" => quote! { .justify_start() },
        "justify-end" => quote! { .justify_end() },
        "justify-center" => quote! { .justify_center() },
        "justify-between" => quote! { .justify_between() },
        "justify-around" => quote! { .justify_around() },
        "justify-evenly" => quote! { .justify_evenly() },
        // Self alignment
        "self-start" => quote! { .self_start() },
        "self-end" => quote! { .self_end() },
        "self-center" => quote! { .self_center() },
        // Position
        "absolute" => quote! { .absolute() },
        "relative" => quote! { .relative() },
        // Overflow
        "overflow-hidden" => quote! { .overflow_hidden() },
        "overflow-auto" => quote! { .overflow_auto() },
        "overflow-scroll" => quote! { .overflow_scroll() },
        "overflow-x-hidden" => quote! { .overflow_x_hidden() },
        "overflow-y-hidden" => quote! { .overflow_y_hidden() },
        // Size
        "w-full" => quote! { .w_full() },
        "w-auto" => quote! { .w_auto() },
        "h-full" => quote! { .h_full() },
        "h-auto" => quote! { .h_auto() },
        "size-full" => quote! { .size_full() },
        "min-h-0" => quote! { .min_h_0() },
        "min-w-0" => quote! { .min_w_0() },
        "max-w-full" => quote! { .max_w_full() },
        "max-h-full" => quote! { .max_h_full() },
        "flex-1" => quote! { .flex_1() },
        // Scrollable overflow (gpui scroll containers)
        "overflow-y-scroll" | "overflow-y-auto" => quote! { .overflow_y_scroll() },
        "overflow-x-scroll" | "overflow-x-auto" => quote! { .overflow_x_scroll() },
        // Absolute offsets (tailwind spacing scale, 1 unit = 4 px)
        "top-0" => quote! { .top(gpui::px(0.0)) },
        "right-0" => quote! { .right(gpui::px(0.0)) },
        "bottom-0" => quote! { .bottom(gpui::px(0.0)) },
        "left-0" => quote! { .left(gpui::px(0.0)) },
        "top-1" => quote! { .top(gpui::px(4.0)) },
        "right-1" => quote! { .right(gpui::px(4.0)) },
        "bottom-1" => quote! { .bottom(gpui::px(4.0)) },
        "left-1" => quote! { .left(gpui::px(4.0)) },
        "top-2" => quote! { .top(gpui::px(8.0)) },
        "right-2" => quote! { .right(gpui::px(8.0)) },
        "bottom-2" => quote! { .bottom(gpui::px(8.0)) },
        "left-2" => quote! { .left(gpui::px(8.0)) },
        "top-3" => quote! { .top(gpui::px(12.0)) },
        "right-3" => quote! { .right(gpui::px(12.0)) },
        "bottom-3" => quote! { .bottom(gpui::px(12.0)) },
        "left-3" => quote! { .left(gpui::px(12.0)) },
        _ => return None,
    })
}

fn transpile_spacing_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Gap
        "gap-0" => quote! { .gap_0() },
        "gap-1" => quote! { .gap(gpui::px(4.0)) },
        "gap-2" => quote! { .gap(gpui::px(8.0)) },
        "gap-3" => quote! { .gap(gpui::px(12.0)) },
        "gap-4" => quote! { .gap(gpui::px(16.0)) },
        "gap-5" => quote! { .gap(gpui::px(20.0)) },
        "gap-6" => quote! { .gap(gpui::px(24.0)) },
        "gap-8" => quote! { .gap(gpui::px(32.0)) },
        "gap-10" => quote! { .gap(gpui::px(40.0)) },
        "gap-12" => quote! { .gap(gpui::px(48.0)) },
        // Padding
        "p-0" => quote! { .p_0() },
        "p-1" => quote! { .p(gpui::px(4.0)) },
        "p-2" => quote! { .p(gpui::px(8.0)) },
        "p-3" => quote! { .p(gpui::px(12.0)) },
        "p-4" => quote! { .p(gpui::px(16.0)) },
        "p-5" => quote! { .p(gpui::px(20.0)) },
        "p-6" => quote! { .p(gpui::px(24.0)) },
        "p-8" => quote! { .p(gpui::px(32.0)) },
        "px-0" => quote! { .px_0() },
        "px-1" => quote! { .px(gpui::px(4.0)) },
        "px-2" => quote! { .px(gpui::px(8.0)) },
        "px-3" => quote! { .px(gpui::px(12.0)) },
        "px-4" => quote! { .px(gpui::px(16.0)) },
        "px-6" => quote! { .px(gpui::px(24.0)) },
        "px-8" => quote! { .px(gpui::px(32.0)) },
        "py-0" => quote! { .py_0() },
        "py-1" => quote! { .py(gpui::px(4.0)) },
        "py-2" => quote! { .py(gpui::px(8.0)) },
        "py-3" => quote! { .py(gpui::px(12.0)) },
        "py-4" => quote! { .py(gpui::px(16.0)) },
        "py-6" => quote! { .py(gpui::px(24.0)) },
        "py-8" => quote! { .py(gpui::px(32.0)) },
        // Margin
        "m-0" => quote! { .m_0() },
        "m-1" => quote! { .m(gpui::px(4.0)) },
        "m-2" => quote! { .m(gpui::px(8.0)) },
        "m-4" => quote! { .m(gpui::px(16.0)) },
        "mx-auto" => quote! { .mx_auto() },
        "mx-0" => quote! { .mx_0() },
        "mx-1" => quote! { .mx(gpui::px(4.0)) },
        "mx-2" => quote! { .mx(gpui::px(8.0)) },
        "mx-3" => quote! { .mx(gpui::px(12.0)) },
        "mx-4" => quote! { .mx(gpui::px(16.0)) },
        "my-0" => quote! { .my_0() },
        "my-1" => quote! { .my(gpui::px(4.0)) },
        "my-2" => quote! { .my(gpui::px(8.0)) },
        "my-3" => quote! { .my(gpui::px(12.0)) },
        "my-4" => quote! { .my(gpui::px(16.0)) },
        "mt-0" => quote! { .mt_0() },
        "mt-1" => quote! { .mt(gpui::px(4.0)) },
        "mt-2" => quote! { .mt(gpui::px(8.0)) },
        "mt-3" => quote! { .mt(gpui::px(12.0)) },
        "mt-4" => quote! { .mt(gpui::px(16.0)) },
        "mb-0" => quote! { .mb_0() },
        "mb-1" => quote! { .mb(gpui::px(4.0)) },
        "mb-2" => quote! { .mb(gpui::px(8.0)) },
        "mb-3" => quote! { .mb(gpui::px(12.0)) },
        "mb-4" => quote! { .mb(gpui::px(16.0)) },
        "ml-0" => quote! { .ml_0() },
        "ml-1" => quote! { .ml(gpui::px(4.0)) },
        "ml-2" => quote! { .ml(gpui::px(8.0)) },
        "ml-3" => quote! { .ml(gpui::px(12.0)) },
        "ml-auto" => quote! { .ml_auto() },
        "mr-0" => quote! { .mr_0() },
        "mr-1" => quote! { .mr(gpui::px(4.0)) },
        "mr-2" => quote! { .mr(gpui::px(8.0)) },
        "mr-3" => quote! { .mr(gpui::px(12.0)) },
        "mr-auto" => quote! { .mr_auto() },
        // Sub-module padding (per-side)
        "pl-0" => quote! { .pl_0() },
        "pl-1" => quote! { .pl(gpui::px(4.0)) },
        "pl-2" => quote! { .pl(gpui::px(8.0)) },
        "pl-3" => quote! { .pl(gpui::px(12.0)) },
        "pl-4" => quote! { .pl(gpui::px(16.0)) },
        "pr-0" => quote! { .pr_0() },
        "pr-1" => quote! { .pr(gpui::px(4.0)) },
        "pr-2" => quote! { .pr(gpui::px(8.0)) },
        "pr-3" => quote! { .pr(gpui::px(12.0)) },
        "pr-4" => quote! { .pr(gpui::px(16.0)) },
        "pt-0" => quote! { .pt_0() },
        "pt-1" => quote! { .pt(gpui::px(4.0)) },
        "pt-2" => quote! { .pt(gpui::px(8.0)) },
        "pt-3" => quote! { .pt(gpui::px(12.0)) },
        "pt-4" => quote! { .pt(gpui::px(16.0)) },
        "pb-0" => quote! { .pb_0() },
        "pb-1" => quote! { .pb(gpui::px(4.0)) },
        "pb-2" => quote! { .pb(gpui::px(8.0)) },
        "pb-3" => quote! { .pb(gpui::px(12.0)) },
        "pb-4" => quote! { .pb(gpui::px(16.0)) },
        "px-5" => quote! { .px(gpui::px(20.0)) },
        "py-5" => quote! { .py(gpui::px(20.0)) },
        "px-1p5" => quote! { .px(gpui::px(6.0)) },
        "py-1p5" => quote! { .py(gpui::px(6.0)) },
        "gap-1p5" => quote! { .gap(gpui::px(6.0)) },
        "p-10" => quote! { .p(gpui::px(40.0)) },
        _ => return None,
    })
}

fn transpile_color_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Opacity
        "opacity-0" => quote! { .opacity(0.0) },
        "opacity-25" => quote! { .opacity(0.25) },
        "opacity-50" => quote! { .opacity(0.5) },
        "opacity-75" => quote! { .opacity(0.75) },
        "opacity-100" => quote! { .opacity(1.0) },
        // Background
        "bg-white" => quote! { .bg(gpui::white()) },
        "bg-black" => quote! { .bg(gpui::black()) },
        "bg-transparent" => quote! { .bg(gpui::transparent_black()) },
        "bg-gray-100" => quote! { .bg(gpui::rgb(0xf3f4f6)) },
        "bg-gray-200" => quote! { .bg(gpui::rgb(0xe5e7eb)) },
        "bg-gray-300" => quote! { .bg(gpui::rgb(0xd1d5db)) },
        "bg-gray-400" => quote! { .bg(gpui::rgb(0x9ca3af)) },
        "bg-gray-500" => quote! { .bg(gpui::rgb(0x6b7280)) },
        "bg-gray-600" => quote! { .bg(gpui::rgb(0x4b5563)) },
        "bg-gray-700" => quote! { .bg(gpui::rgb(0x374151)) },
        "bg-gray-800" => quote! { .bg(gpui::rgb(0x1f2937)) },
        "bg-gray-900" => quote! { .bg(gpui::rgb(0x111827)) },
        "bg-gray-950" => quote! { .bg(gpui::rgb(0x030712)) },
        "bg-red-500" => quote! { .bg(gpui::rgb(0xef4444)) },
        "bg-red-600" => quote! { .bg(gpui::rgb(0xdc2626)) },
        "bg-blue-500" => quote! { .bg(gpui::rgb(0x3b82f6)) },
        "bg-blue-600" => quote! { .bg(gpui::rgb(0x2563eb)) },
        "bg-green-500" => quote! { .bg(gpui::rgb(0x22c55e)) },
        "bg-green-600" => quote! { .bg(gpui::rgb(0x16a34a)) },
        "bg-purple-500" => quote! { .bg(gpui::rgb(0xa855f7)) },
        "bg-purple-600" => quote! { .bg(gpui::rgb(0x9333ea)) },
        "bg-yellow-500" => quote! { .bg(gpui::rgb(0xeab308)) },
        "bg-yellow-600" => quote! { .bg(gpui::rgb(0xca8a04)) },
        // Text color
        "text-white" => quote! { .text_color(gpui::white()) },
        "text-black" => quote! { .text_color(gpui::black()) },
        "text-gray-300" => quote! { .text_color(gpui::rgb(0xd1d5db)) },
        "text-gray-400" => quote! { .text_color(gpui::rgb(0x9ca3af)) },
        "text-gray-500" => quote! { .text_color(gpui::rgb(0x6b7280)) },
        "text-gray-600" => quote! { .text_color(gpui::rgb(0x4b5563)) },
        "text-gray-700" => quote! { .text_color(gpui::rgb(0x374151)) },
        "text-gray-900" => quote! { .text_color(gpui::rgb(0x111827)) },
        "text-blue-500" => quote! { .text_color(gpui::rgb(0x3b82f6)) },
        "text-blue-600" => quote! { .text_color(gpui::rgb(0x2563eb)) },
        "text-red-500" => quote! { .text_color(gpui::rgb(0xef4444)) },
        "text-red-600" => quote! { .text_color(gpui::rgb(0xdc2626)) },
        "text-green-500" => quote! { .text_color(gpui::rgb(0x22c55e)) },
        "text-green-600" => quote! { .text_color(gpui::rgb(0x16a34a)) },
        _ => return None,
    })
}

fn transpile_typography_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Font size (text-* colors live in `transpile_color_class`)
        "text-xs" => quote! { .text_xs() },
        "text-sm" => quote! { .text_sm() },
        "text-base" => quote! { .text_base() },
        "text-lg" => quote! { .text_lg() },
        "text-xl" => quote! { .text_xl() },
        "text-2xl" => quote! { .text_2xl() },
        "text-3xl" => quote! { .text_3xl() },
        "text-4xl" => quote! { .text_4xl() },
        // Font weight
        "font-thin" => quote! { .font_weight(gpui::FontWeight::THIN) },
        "font-light" => quote! { .font_weight(gpui::FontWeight::LIGHT) },
        "font-normal" => quote! { .font_weight(gpui::FontWeight::NORMAL) },
        "font-medium" => quote! { .font_weight(gpui::FontWeight::MEDIUM) },
        "font-semibold" => quote! { .font_weight(gpui::FontWeight::SEMIBOLD) },
        "font-bold" => quote! { .font_weight(gpui::FontWeight::BOLD) },
        // Whitespace / text alignment
        "whitespace-nowrap" => quote! { .whitespace_nowrap() },
        "truncate" => quote! { .truncate() },
        "text-center" => quote! { .text_center() },
        "text-left" => quote! { .text_left() },
        "text-right" => quote! { .text_right() },
        "select-none" => quote! { .select_none() },
        // Font families
        "font-mono" => quote! { .font_family("monospace") },
        "font-sans" => quote! { .font_family("sans-serif") },
        _ => return None,
    })
}

fn transpile_border_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Border widths
        "border" => quote! { .border_1() },
        "border-0" => quote! { .border_0() },
        "border-2" => quote! { .border_2() },
        // Border colors
        "border-gray-200" => quote! { .border_color(gpui::rgb(0xe5e7eb)) },
        "border-gray-300" => quote! { .border_color(gpui::rgb(0xd1d5db)) },
        "border-gray-500" => quote! { .border_color(gpui::rgb(0x6b7280)) },
        "border-gray-700" => quote! { .border_color(gpui::rgb(0x374151)) },
        "border-blue-500" => quote! { .border_color(gpui::rgb(0x3b82f6)) },
        "border-red-500" => quote! { .border_color(gpui::rgb(0xef4444)) },
        "border-white" => quote! { .border_color(gpui::white()) },
        "border-black" => quote! { .border_color(gpui::black()) },
        // Per-side borders
        "border-b" => quote! { .border_b_1() },
        "border-b-0" => quote! { .border_b_0() },
        "border-b-2" => quote! { .border_b_2() },
        "border-t" => quote! { .border_t_1() },
        "border-t-0" => quote! { .border_t_0() },
        "border-l" => quote! { .border_l_1() },
        "border-r" => quote! { .border_r_1() },
        // Border radius
        "rounded-none" => quote! { .rounded(gpui::px(0.0)) },
        "rounded-sm" => quote! { .rounded(gpui::px(2.0)) },
        "rounded" => quote! { .rounded(gpui::px(4.0)) },
        "rounded-md" => quote! { .rounded(gpui::px(6.0)) },
        "rounded-lg" => quote! { .rounded(gpui::px(8.0)) },
        "rounded-xl" => quote! { .rounded(gpui::px(12.0)) },
        "rounded-2xl" => quote! { .rounded(gpui::px(16.0)) },
        "rounded-full" => quote! { .rounded(gpui::px(9999.0)) },
        // Per-corner radii
        "rounded-tl" => quote! { .rounded_tl(gpui::px(4.0)) },
        "rounded-tr" => quote! { .rounded_tr(gpui::px(4.0)) },
        "rounded-bl" => quote! { .rounded_bl(gpui::px(4.0)) },
        "rounded-br" => quote! { .rounded_br(gpui::px(4.0)) },
        "rounded-tl-lg" => quote! { .rounded_tl(gpui::px(8.0)) },
        "rounded-tr-lg" => quote! { .rounded_tr(gpui::px(8.0)) },
        "rounded-bl-lg" => quote! { .rounded_bl(gpui::px(8.0)) },
        "rounded-br-lg" => quote! { .rounded_br(gpui::px(8.0)) },
        "rounded-tl-md" => quote! { .rounded_tl(gpui::px(6.0)) },
        "rounded-tl-xl" => quote! { .rounded_tl(gpui::px(12.0)) },
        // Box shadows
        "shadow-xs" => quote! { .shadow_xs() },
        "shadow-sm" => quote! { .shadow_sm() },
        "shadow-md" => quote! { .shadow_md() },
        "shadow-lg" => quote! { .shadow_lg() },
        "shadow-xl" => quote! { .shadow_xl() },
        _ => return None,
    })
}

fn transpile_misc_class(class: &str) -> Option<TokenStream> {
    Some(match class {
        // Cursor
        "cursor-pointer" => quote! { .cursor_pointer() },
        "cursor-default" => quote! { .cursor_default() },
        "cursor-text" => quote! { .cursor_text() },
        _ => return None,
    })
}

/// Parses a tailwind arbitrary value (the part between the brackets).
/// Returns the value string without the trailing `px` unit, when present.
fn arbitrary_px_value(inner: &str) -> Option<f32> {
    let value = inner.strip_suffix("px").unwrap_or(inner);
    value.parse::<f32>().ok()
}

/// Parses `#rrggbb` or `#rrggbbaa` into a `u32`/alpha pair.
fn arbitrary_hex_color(inner: &str) -> Option<(u32, f32)> {
    let hex = inner.strip_prefix('#')?;
    let (rgb, alpha) = match hex.len() {
        6 => (u32::from_str_radix(hex, 16).ok()?, 1.0),
        8 => (
            u32::from_str_radix(&hex[..6], 16).ok()?,
            f32::from(u8::from_str_radix(&hex[6..], 16).ok()?) / 255.0,
        ),
        _ => return None,
    };
    Some((rgb, alpha))
}

/// Tailwind arbitrary colors: `bg-[#rrggbb]`, `text-[#rrggbbaa]`,
/// `border-[#rrggbb]`. Kept separate so numeric arbitrary values and color
/// values never collide.
fn transpile_arbitrary_color_class(class: &str) -> Option<TokenStream> {
    let (name, inner) = class.split_once('[')?;
    let inner = inner.strip_suffix(']')?;
    let (rgb, alpha) = arbitrary_hex_color(inner)?;
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha_byte = (alpha * 255.0).round() as u32;
    let rgba_u32 = (rgb << 8) | alpha_byte;
    Some(match name {
        "bg" if alpha >= 1.0 => quote! { .bg(gpui::rgb(#rgb)) },
        "bg" => quote! { .bg(gpui::rgba(#rgba_u32)) },
        "text" if alpha >= 1.0 => quote! { .text_color(gpui::rgb(#rgb)) },
        "text" => quote! { .text_color(gpui::rgba(#rgba_u32)) },
        "border" if alpha >= 1.0 => quote! { .border_color(gpui::rgb(#rgb)) },
        "border" => quote! { .border_color(gpui::rgba(#rgba_u32)) },
        _ => return None,
    })
}

fn transpile_single_class(class: &str) -> Option<TokenStream> {
    transpile_layout_class(class)
        .or_else(|| transpile_spacing_class(class))
        .or_else(|| transpile_color_class(class))
        .or_else(|| transpile_typography_class(class))
        .or_else(|| transpile_border_class(class))
        .or_else(|| transpile_misc_class(class))
        // Tailwind arbitrary values (`w-[600px]`, `text-[11px]`, …).
        .or_else(|| transpile_arbitrary_px_class(class))
        // Tailwind arbitrary colors (`bg-[#1e2e3e]`, …).
        .or_else(|| transpile_arbitrary_color_class(class))
        // Resolves a shadcn theme token (e.g. "text-primary",
        // "bg-destructive") as the last try. Inlined as a closure on
        // purpose: a standalone `fn` here would be suggested as `const` by
        // clippy, but the `gpui` feature path calls the non-const
        // `try_resolve_gpui_color`, so `const` would break the build.
        .or_else(|| {
            #[cfg(feature = "gpui")]
            if let Some(color_token) = crate::transpile::theme_tokens::try_resolve_gpui_color(class)
            {
                return Some(color_token);
            }
            let _ = class;
            None
        })
}

/// Arbitrary pixel-value classes (`p-[13px]`, `w-[600px]`, `top-[100px]`,
/// `opacity-[0.9]`, `text-[11px]`, …). The `px` suffix is optional so bare
/// numbers work too.
fn transpile_arbitrary_px_class(class: &str) -> Option<TokenStream> {
    let (name, inner) = class.split_once('[')?;
    let inner = inner.strip_suffix(']')?;
    // Colors are handled by [`transpile_arbitrary_color_class`].
    if inner.starts_with('#') {
        return None;
    }
    let px = arbitrary_px_value(inner)?;
    let px_lit = proc_macro2::Literal::f32_suffixed(px);
    Some(match name {
        "p" => quote! { .p(gpui::px(#px_lit)) },
        "px" => quote! { .px(gpui::px(#px_lit)) },
        "py" => quote! { .py(gpui::px(#px_lit)) },
        "pt" => quote! { .pt(gpui::px(#px_lit)) },
        "pb" => quote! { .pb(gpui::px(#px_lit)) },
        "pl" => quote! { .pl(gpui::px(#px_lit)) },
        "pr" => quote! { .pr(gpui::px(#px_lit)) },
        "m" => quote! { .m(gpui::px(#px_lit)) },
        "mx" => quote! { .mx(gpui::px(#px_lit)) },
        "my" => quote! { .my(gpui::px(#px_lit)) },
        "mt" => quote! { .mt(gpui::px(#px_lit)) },
        "mb" => quote! { .mb(gpui::px(#px_lit)) },
        "ml" => quote! { .ml(gpui::px(#px_lit)) },
        "mr" => quote! { .mr(gpui::px(#px_lit)) },
        "gap" => quote! { .gap(gpui::px(#px_lit)) },
        "top" => quote! { .top(gpui::px(#px_lit)) },
        "right" => quote! { .right(gpui::px(#px_lit)) },
        "bottom" => quote! { .bottom(gpui::px(#px_lit)) },
        "left" => quote! { .left(gpui::px(#px_lit)) },
        "w" => quote! { .w(gpui::px(#px_lit)) },
        "h" => quote! { .h(gpui::px(#px_lit)) },
        "min-w" => quote! { .min_w(gpui::px(#px_lit)) },
        "min-h" => quote! { .min_h(gpui::px(#px_lit)) },
        "max-w" => quote! { .max_w(gpui::px(#px_lit)) },
        "max-h" => quote! { .max_h(gpui::px(#px_lit)) },
        "rounded" => quote! { .rounded(gpui::px(#px_lit)) },
        "rounded-tl" => quote! { .rounded_tl(gpui::px(#px_lit)) },
        "text" => quote! { .text_size(gpui::px(#px_lit)) },
        "opacity" => quote! { .opacity(#px_lit) },
        _ => return None,
    })
}
