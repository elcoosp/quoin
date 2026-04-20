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
            "hidden" => quote! { .hidden() },
            // Flex direction
            "flex-col" => quote! { .flex_col() },
            "flex-row" => quote! { .flex_row() },
            // Alignment
            "items-center" => quote! { .items_center() },
            "items-start" => quote! { .items_start() },
            "items-end" => quote! { .items_end() },
            "justify-center" => quote! { .justify_center() },
            "justify-between" => quote! { .justify_between() },
            "justify-start" => quote! { .justify_start() },
            "justify-end" => quote! { .justify_end() },
            // Spacing
            "gap-0" => quote! { .gap_0() },
            "gap-1" => quote! { .gap_1() },
            "gap-2" => quote! { .gap_2() },
            "gap-3" => quote! { .gap_3() },
            "gap-4" => quote! { .gap_4() },
            "gap-6" => quote! { .gap_6() },
            "gap-8" => quote! { .gap_8() },
            // Padding
            "p-0" => quote! { .p_0() },
            "p-1" => quote! { .p_1() },
            "p-2" => quote! { .p_2() },
            "p-3" => quote! { .p_3() },
            "p-4" => quote! { .p_4() },
            "px-2" => quote! { .px_2() },
            "px-4" => quote! { .px_4() },
            "py-1" => quote! { .py_1() },
            "py-2" => quote! { .py_2() },
            // Sizing
            "w-full" => quote! { .w_full() },
            "h-full" => quote! { .h_full() },
            "size-full" => quote! { .size_full() },
            // Background
            "bg-white" => quote! { .bg(gpui::white()) },
            "bg-black" => quote! { .bg(gpui::black()) },
            "bg-gray-100" => quote! { .bg(gpui::rgb(0xf3f4f6)) },
            "bg-gray-800" => quote! { .bg(gpui::rgb(0x1f2937)) },
            "bg-gray-900" => quote! { .bg(gpui::rgb(0x111827)) },
            "bg-blue-600" => quote! { .bg(gpui::rgb(0x2563eb)) },
            "bg-green-600" => quote! { .bg(gpui::rgb(0x16a34a)) },
            "bg-purple-600" => quote! { .bg(gpui::rgb(0x9333ea)) },
            // Text color
            "text-white" => quote! { .text_color(gpui::white()) },
            "text-black" => quote! { .text_color(gpui::black()) },
            // Font size
            "text-xs" => quote! { .text_xs() },
            "text-sm" => quote! { .text_sm() },
            "text-base" => quote! { .text_base() },
            "text-lg" => quote! { .text_lg() },
            "text-xl" => quote! { .text_xl() },
            "text-2xl" => quote! { .text_2xl() },
            // Border radius
            "rounded" => quote! { .rounded(gpui::px(4.0)) },
            "rounded-sm" => quote! { .rounded(gpui::px(2.0)) },
            "rounded-md" => quote! { .rounded(gpui::px(6.0)) },
            "rounded-lg" => quote! { .rounded(gpui::px(8.0)) },
            "rounded-full" => quote! { .rounded(gpui::px(9999.0)) },
            // Cursor
            "cursor-pointer" => quote! { .cursor_pointer() },
            "cursor-default" => quote! { .cursor_default() },
            _ => continue,
        };
        tokens.push(token);
    }
    tokens
}
