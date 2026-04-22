//! Inline SVG icon mapping for the ~15 icons used in the Devtools.
//!
//! Provides two APIs:
//! - [`icon_svg_html`] — complete SVG HTML string (for Leptos `inner_html`)
//! - [`icon_children`] — structured SVG child elements (for Dioxus rsx! construction)

const SVG_ATTRS: &str = r#"xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round""#;

/// A single SVG child element (path, circle, rect, polygon, etc.)
pub struct SvgChild {
    pub tag: &'static str,
    pub attrs: Vec<(&'static str, &'static str)>,
}

/// Map an icon name to a complete SVG HTML string.
/// Returns None for unknown icons (caller should fall back to a placeholder).
pub fn icon_svg_html(name: &str) -> Option<String> {
    let children = get_children(name)?;
    let body: String = children.iter().map(|c| {
        let attrs: String = c.attrs.iter().map(|(k, v)| format!("{}=\"{}\"", k, v)).collect::<Vec<_>>().join(" ");
        format!("<{} {}/>", c.tag, attrs)
    }).collect();
    Some(format!("<svg {}>{}</svg>", SVG_ATTRS, body))
}

/// Map an icon name to structured SVG child elements for rsx! construction.
pub fn icon_children(name: &str) -> Option<Vec<SvgChild>> {
    get_children(name)
}

fn get_children(name: &str) -> Option<Vec<SvgChild>> {
    Some(match name {
        "calendar" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M8 2v4")] },
            SvgChild { tag: "path", attrs: vec![("d", "M16 2v4")] },
            SvgChild { tag: "rect", attrs: vec![("width", "18"), ("height", "18"), ("x", "3"), ("y", "4"), ("rx", "2")] },
            SvgChild { tag: "path", attrs: vec![("d", "M3 10h18")] },
        ],
        "info" => vec![
            SvgChild { tag: "circle", attrs: vec![("cx", "12"), ("cy", "12"), ("r", "10")] },
            SvgChild { tag: "path", attrs: vec![("d", "M12 16v-4")] },
            SvgChild { tag: "path", attrs: vec![("d", "M12 8h.01")] },
        ],
        "search" => vec![
            SvgChild { tag: "circle", attrs: vec![("cx", "11"), ("cy", "11"), ("r", "8")] },
            SvgChild { tag: "path", attrs: vec![("d", "m21 21-4.3-4.3")] },
        ],
        "close" | "x" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M18 6 6 18")] },
            SvgChild { tag: "path", attrs: vec![("d", "m6 6 12 12")] },
        ],
        "trash" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M3 6h18")] },
            SvgChild { tag: "path", attrs: vec![("d", "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6")] },
            SvgChild { tag: "path", attrs: vec![("d", "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2")] },
        ],
        "copy" => vec![
            SvgChild { tag: "rect", attrs: vec![("width", "14"), ("height", "14"), ("x", "8"), ("y", "8"), ("rx", "2"), ("ry", "2")] },
            SvgChild { tag: "path", attrs: vec![("d", "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2")] },
        ],
        "check" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M20 6 9 17l-5-5")] },
        ],
        "chevron-down" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "m6 9 6 6 6-6")] },
        ],
        "chevron-right" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "m9 18 6-6-6-6")] },
        ],
        "folder" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z")] },
        ],
        "inbox" => vec![
            SvgChild { tag: "polyline", attrs: vec![("points", "22 12 16 12 14 15 10 15 8 12 2 12")] },
            SvgChild { tag: "path", attrs: vec![("d", "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z")] },
        ],
        "settings" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z")] },
            SvgChild { tag: "circle", attrs: vec![("cx", "12"), ("cy", "12"), ("r", "3")] },
        ],
        "file" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z")] },
            SvgChild { tag: "path", attrs: vec![("d", "M14 2v4a2 2 0 0 0 2 2h4")] },
        ],
        "play" => vec![
            SvgChild { tag: "polygon", attrs: vec![("points", "6 3 20 12 6 21 6 3")] },
        ],
        "map" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M14.106 5.553a2 2 0 0 0 1.788 0l3.659-1.83A1 1 0 0 1 21 4.619v12.764a1 1 0 0 1-.553.894l-4.553 2.277a2 2 0 0 1-1.788 0l-4.212-2.106a2 2 0 0 0-1.788 0l-3.659 1.83A1 1 0 0 1 3 19.381V6.618a1 1 0 0 1 .553-.894l4.553-2.277a2 2 0 0 1 1.788 0z")] },
            SvgChild { tag: "path", attrs: vec![("d", "M15 5.764v15")] },
            SvgChild { tag: "path", attrs: vec![("d", "M9 3.236v15")] },
        ],
        "loader" | "refresh" => vec![
            SvgChild { tag: "path", attrs: vec![("d", "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74")] },
            SvgChild { tag: "path", attrs: vec![("d", "M21 3v6h-6")] },
        ],
        _ => return None,
    })
}
