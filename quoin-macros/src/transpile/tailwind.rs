use proc_macro2::TokenStream;
use quote::quote;

pub fn transpile_class(class_str: &str) -> Vec<TokenStream> {
    let mut tokens = Vec::new();
    for class in class_str.split_whitespace() {
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
            // Gap (Tailwind spacing scale: 1 = 0.25rem = 4px in GPUI)
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
            "px-1" => quote! { .px(gpui::px(4.0)) },
            "px-2" => quote! { .px(gpui::px(8.0)) },
            "px-3" => quote! { .px(gpui::px(12.0)) },
            "px-4" => quote! { .px(gpui::px(16.0)) },
            "py-1" => quote! { .py(gpui::px(4.0)) },
            "py-2" => quote! { .py(gpui::px(8.0)) },
            // Margin
            "m-1" => quote! { .m(gpui::px(4.0)) },
            "m-2" => quote! { .m(gpui::px(8.0)) },
            "m-4" => quote! { .m(gpui::px(16.0)) },
            "mx-auto" => quote! { .mx_auto() },
            "my-2" => quote! { .my(gpui::px(8.0)) },
            // Width / Height
            "w-full" => quote! { .w_full() },
            "w-auto" => quote! { .w_auto() },
            "h-full" => quote! { .h_full() },
            "size-full" => quote! { .size_full() },
            "w-1/2" => quote! { .w_1of2() },
            "w-1/3" => quote! { .w_1of3() },
            // Background
            "bg-white" => quote! { .bg(gpui::white()) },
            "bg-black" => quote! { .bg(gpui::black()) },
            "bg-gray-100" => quote! { .bg(gpui::rgb(0xf3f4f6)) },
            "bg-gray-800" => quote! { .bg(gpui::rgb(0x1f2937)) },
            "bg-gray-900" => quote! { .bg(gpui::rgb(0x111827)) },
            "bg-blue-500" => quote! { .bg(gpui::rgb(0x3b82f6)) },
            "bg-blue-600" => quote! { .bg(gpui::rgb(0x2563eb)) },
            "bg-green-600" => quote! { .bg(gpui::rgb(0x16a34a)) },
            "bg-purple-600" => quote! { .bg(gpui::rgb(0x9333ea)) },
            // Text color
            "text-white" => quote! { .text_color(gpui::white()) },
            "text-black" => quote! { .text_color(gpui::black()) },
            "text-gray-400" => quote! { .text_color(gpui::rgb(0x9ca3af)) },
            // Font size
            "text-xs" => quote! { .text_xs() },
            "text-sm" => quote! { .text_sm() },
            "text-base" => quote! { .text_base() },
            "text-lg" => quote! { .text_lg() },
            "text-xl" => quote! { .text_xl() },
            "text-2xl" => quote! { .text_2xl() },
            "text-3xl" => quote! { .text_3xl() },
            // Font weight
            "font-thin" => quote! { .font_weight(gpui::FontWeight::THIN) },
            "font-light" => quote! { .font_weight(gpui::FontWeight::LIGHT) },
            "font-normal" => quote! { .font_weight(gpui::FontWeight::NORMAL) },
            "font-medium" => quote! { .font_weight(gpui::FontWeight::MEDIUM) },
            "font-semibold" => quote! { .font_weight(gpui::FontWeight::SEMIBOLD) },
            "font-bold" => quote! { .font_weight(gpui::FontWeight::BOLD) },
            // Border radius (all require pixel argument)
            "rounded-none" => quote! { .rounded(gpui::px(0.0)) },
            "rounded-sm" => quote! { .rounded(gpui::px(2.0)) },
            "rounded" => quote! { .rounded(gpui::px(4.0)) },
            "rounded-md" => quote! { .rounded(gpui::px(6.0)) },
            "rounded-lg" => quote! { .rounded(gpui::px(8.0)) },
            "rounded-xl" => quote! { .rounded(gpui::px(12.0)) },
            "rounded-full" => quote! { .rounded(gpui::px(9999.0)) },
            // Cursor
            "cursor-pointer" => quote! { .cursor_pointer() },
            "cursor-default" => quote! { .cursor_default() },
            // Position
            "absolute" => quote! { .absolute() },
            "relative" => quote! { .relative() },
            // Overflow
            "overflow-hidden" => quote! { .overflow_hidden() },
            "overflow-auto" => quote! { .overflow_auto() },
            "overflow-scroll" => quote! { .overflow_scroll() },
            _ => continue,
        };
        tokens.push(token);
    }
    tokens
}
