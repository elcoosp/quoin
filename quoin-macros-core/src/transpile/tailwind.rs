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

pub struct TranspiledStyles {
    pub normal: Vec<TokenStream>,
    pub hover: Vec<TokenStream>,
}

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

fn transpile_single_class(class: &str) -> Option<TokenStream> {
    let token = match class {
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
        "flex-grow" => quote! { .flex_grow(1.0) },
        "flex-grow-0" => quote! { .flex_grow(0.0) },
        "flex-shrink" => quote! { .flex_shrink(1.0) },
        "flex-shrink-0" => quote! { .flex_shrink(0.0) },
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
        "mx-2" => quote! { .mx(gpui::px(8.0)) },
        "mx-4" => quote! { .mx(gpui::px(16.0)) },
        "my-0" => quote! { .my_0() },
        "my-2" => quote! { .my(gpui::px(8.0)) },
        "my-4" => quote! { .my(gpui::px(16.0)) },
        // Width / Height
        "w-full" => quote! { .w_full() },
        "w-auto" => quote! { .w_auto() },
        "h-full" => quote! { .h_full() },
        "h-auto" => quote! { .h_auto() },
        "size-full" => quote! { .size_full() },
        "min-h-0" => quote! { .min_h_0() },
        "min-w-0" => quote! { .min_w_0() },
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
        // Font size
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
        // Border radius
        "rounded-none" => quote! { .rounded(gpui::px(0.0)) },
        "rounded-sm" => quote! { .rounded(gpui::px(2.0)) },
        "rounded" => quote! { .rounded(gpui::px(4.0)) },
        "rounded-md" => quote! { .rounded(gpui::px(6.0)) },
        "rounded-lg" => quote! { .rounded(gpui::px(8.0)) },
        "rounded-xl" => quote! { .rounded(gpui::px(12.0)) },
        "rounded-2xl" => quote! { .rounded(gpui::px(16.0)) },
        "rounded-full" => quote! { .rounded(gpui::px(9999.0)) },
        // Cursor
        "cursor-pointer" => quote! { .cursor_pointer() },
        "cursor-default" => quote! { .cursor_default() },
        "cursor-text" => quote! { .cursor_text() },
        // Position
        "absolute" => quote! { .absolute() },
        "relative" => quote! { .relative() },
        // Overflow
        "overflow-hidden" => quote! { .overflow_hidden() },
        "overflow-auto" => quote! { .overflow_auto() },
        "overflow-scroll" => quote! { .overflow_scroll() },
        "overflow-x-hidden" => quote! { .overflow_x_hidden() },
        "overflow-y-hidden" => quote! { .overflow_y_hidden() },
        // Whitespace / text
        "whitespace-nowrap" => quote! { .whitespace_nowrap() },
        "truncate" => quote! { .truncate() },
        "text-center" => quote! { .text_center() },
        "text-left" => quote! { .text_left() },
        "text-right" => quote! { .text_right() },
        // Select
        "select-none" => quote! { .select_none() },
        // Border
        "border" => quote! { .border_1() },
        "border-0" => quote! { .border_0() },
        "border-2" => quote! { .border_2() },
        // Border color
        "border-gray-200" => quote! { .border_color(gpui::rgb(0xe5e7eb)) },
        "border-gray-300" => quote! { .border_color(gpui::rgb(0xd1d5db)) },
        "border-gray-500" => quote! { .border_color(gpui::rgb(0x6b7280)) },
        "border-gray-700" => quote! { .border_color(gpui::rgb(0x374151)) },
        "border-blue-500" => quote! { .border_color(gpui::rgb(0x3b82f6)) },
        "border-red-500" => quote! { .border_color(gpui::rgb(0xef4444)) },
        "border-white" => quote! { .border_color(gpui::white()) },
        "border-black" => quote! { .border_color(gpui::black()) },
        // Top/right/bottom/left borders
        "border-b" => quote! { .border_b_1() },
        "border-b-0" => quote! { .border_b_0() },
        "border-b-2" => quote! { .border_b_2() },
        "border-t" => quote! { .border_t_1() },
        "border-t-0" => quote! { .border_t_0() },
        "border-l" => quote! { .border_l_1() },
        "border-r" => quote! { .border_r_1() },
        _ => return None,
    };
    Some(token)
}
