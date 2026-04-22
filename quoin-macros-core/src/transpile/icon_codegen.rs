//! Inline SVG icon mapping for the ~15 icons used in the Devtools.
//!
//! When shadcn + lucide-* crate deps are available, this module will be
//! upgraded to emit the actual component calls. Until then, we emit
//! inline SVGs (lucide 24x24 style, stroke-based).

use proc_macro2::TokenStream;
use quote::quote;

/// Map an icon name string to an inline SVG TokenStream.
/// Returns None for unknown icons (caller should fall back to a placeholder).
pub fn icon_to_svg(name: &str) -> Option<TokenStream> {
    let path = match name {
        "calendar" => "M8 2v14a6 6 0 0 1 12 6 6 0 0 1-12 0V2m2 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2M8 4v12m0-18a2 2 0 0 0-2 2h-2a2 2 0 0 0-2-2v14a2 2 0 0 0 2 2h2a2 2 0 0 0 2 2",
        "info" => "M2 12a10 10 0 1 0 20 0 10 10 0 1 0-20 0M12 16v-4m-6 0h-4m0 0h6M2 12a10 10 0 1 0 20 0 10 10 0 1 0-20 0",
        "search" => "m21 21-5.197-2.132-3.898-4.378-4.122-4.122-5.379 0-7.21 2.048-8.49 4.886-9.757 4.886-9.757 0-2.390-0.866-4.378-2.048-8.49",
        "close" | "x" => "M18 6 6 6 0 1-12 0 6 6 0 0 1 12 0M6 18 6 6 0 0 1-12 0 6 6 0 0 1 12 0M6 6 0 0 1-12 0",
        "trash" => "M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m0 0a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2M10 14a2 2 0 1 1-4 0 2 2 0 0 1 4 0",
        "copy" => "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2m10 0a2 2 0 0 1-4 0 2 2 0 0 1 4 0",
        "check" => "M20 6 9a2 2 0 0 0-2 2H6a2 2 0 0 0-2-2v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2 2m-2 4a2 2 0 0 0-2-2h-2a2 2 0 0 0-2 2h12a2 2 0 0 0 2 2",
        "chevron-down" => "m6 9 6 6 0 1 1 12 0 6 6 0 0 1-12 0",
        "chevron-right" => "m9 18 6-6-6 0 1 1 12 0 6 6 0 0 1-12 0",
        "folder" => "M4 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h2l2-2v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2 2V8l-2 2",
        "inbox" => "M4 4h16c1.1 0 2 .9 2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V4m0 0h16M4 4v16M4 4h16",
        "settings" => "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 0 2 2h.44M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2M2 12a10 10 0 1 0 20 0 10 10 0 1 0-20 0",
        "file" => "M15 2H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V4M14 2v4a2 2 0 0 0-2 2H4",
        "play" => "m5 3 4 0 1 0 8 0 4 0 1 0 0-8zM1 7a4 4 0 1 1 0-8 0 4 4 0 0 1 0 8",
        "map" => "M3 12l2-2m0 0h6m6 0h6m-6 0a2 2 0 1 0 4 0 2 2 0 0 0-4 0m-6 4V6",
        "loader" | "refresh" => "M4 4v5h.582a15 15 0 0 1 30 0 15 15 0 0 1-30 0V4a2 2 0 0 1 2-2h18a2 2 0 0 1 2 2M4 4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h18a2 2 0 0 0 2-2V4",
        _ => return None,
    };

    Some(quote! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        ><path d={#path} /></svg>
    })
}
