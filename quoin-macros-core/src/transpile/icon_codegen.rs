//! Inline SVG icon mapping for the ~15 icons used in the Devtools.
//!
//! The icons live in one declarative [`ICON_TABLE`] (lucide-style paths); two
//! accessors render them:
//! - [`icon_svg_html`] — complete SVG HTML string (for Leptos `inner_html`)
//! - [`icon_children`] — structured SVG child elements (for Dioxus rsx!
//!   construction)
//!
//! Returns `None` for unknown icons (the caller falls back to a placeholder).

use std::fmt::Write as _;

const SVG_ATTRS: &str = r#"xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round""#;

/// A single SVG child element (path, circle, rect, polygon, etc.)
#[derive(Debug, Clone, Copy)]
pub struct SvgChild {
    /// Element tag: `path`, `circle`, `rect`, `polyline`, …
    pub tag: &'static str,
    /// Attribute pairs, e.g. `("d", "M8 2v4")`.
    pub attrs: &'static [(&'static str, &'static str)],
}

/// One icon definition: the names it answers to plus its shape children.
struct IconDef {
    names: &'static [&'static str],
    children: &'static [SvgChild],
}

const fn child(tag: &'static str, attrs: &'static [(&'static str, &'static str)]) -> SvgChild {
    SvgChild { tag, attrs }
}

/// Declarative icon table: lucide-style 24×24 stroke icons.
static ICON_TABLE: &[IconDef] = &[
    IconDef {
        names: &["calendar"],
        children: &[
            child("path", &[("d", "M8 2v4")]),
            child("path", &[("d", "M16 2v4")]),
            child(
                "rect",
                &[
                    ("width", "18"),
                    ("height", "18"),
                    ("x", "3"),
                    ("y", "4"),
                    ("rx", "2"),
                ],
            ),
            child("path", &[("d", "M3 10h18")]),
        ],
    },
    IconDef {
        names: &["info"],
        children: &[
            child("circle", &[("cx", "12"), ("cy", "12"), ("r", "10")]),
            child("path", &[("d", "M12 16v-4")]),
            child("path", &[("d", "M12 8h.01")]),
        ],
    },
    IconDef {
        names: &["search"],
        children: &[
            child("circle", &[("cx", "11"), ("cy", "11"), ("r", "8")]),
            child("path", &[("d", "m21 21-4.3-4.3")]),
        ],
    },
    IconDef {
        names: &["close", "x"],
        children: &[
            child("path", &[("d", "M18 6 6 18")]),
            child("path", &[("d", "m6 6 12 12")]),
        ],
    },
    IconDef {
        names: &["trash"],
        children: &[
            child("path", &[("d", "M3 6h18")]),
            child("path", &[("d", "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6")]),
            child("path", &[("d", "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2")]),
        ],
    },
    IconDef {
        names: &["copy"],
        children: &[
            child(
                "rect",
                &[
                    ("width", "14"),
                    ("height", "14"),
                    ("x", "8"),
                    ("y", "8"),
                    ("rx", "2"),
                    ("ry", "2"),
                ],
            ),
            child(
                "path",
                &[(
                    "d",
                    "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
                )],
            ),
        ],
    },
    IconDef {
        names: &["check"],
        children: &[child("path", &[("d", "M20 6 9 17l-5-5")])],
    },
    IconDef {
        names: &["chevron-down"],
        children: &[child("path", &[("d", "m6 9 6 6 6-6")])],
    },
    IconDef {
        names: &["chevron-right"],
        children: &[child("path", &[("d", "m9 18 6-6-6-6")])],
    },
    IconDef {
        names: &["folder"],
        children: &[child(
            "path",
            &[(
                "d",
                "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
            )],
        )],
    },
    IconDef {
        names: &["inbox"],
        children: &[
            child(
                "polyline",
                &[("points", "22 12 16 12 14 15 10 15 8 12 2 12")],
            ),
            child(
                "path",
                &[(
                    "d",
                    "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z",
                )],
            ),
        ],
    },
    IconDef {
        names: &["settings"],
        children: &[
            child(
                "path",
                &[(
                    "d",
                    "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
                )],
            ),
            child("circle", &[("cx", "12"), ("cy", "12"), ("r", "3")]),
        ],
    },
    IconDef {
        names: &["file"],
        children: &[
            child(
                "path",
                &[(
                    "d",
                    "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z",
                )],
            ),
            child("path", &[("d", "M14 2v4a2 2 0 0 0 2 2h4")]),
        ],
    },
    IconDef {
        names: &["play"],
        children: &[child("polygon", &[("points", "6 3 20 12 6 21 6 3")])],
    },
    IconDef {
        names: &["map"],
        children: &[
            child(
                "path",
                &[(
                    "d",
                    "M14.106 5.553a2 2 0 0 0 1.788 0l3.659-1.83A1 1 0 0 1 21 4.619v12.764a1 1 0 0 1-.553.894l-4.553 2.277a2 2 0 0 1-1.788 0l-4.212-2.106a2 2 0 0 0-1.788 0l-3.659 1.83A1 1 0 0 1 3 19.381V6.618a1 1 0 0 1 .553-.894l4.553-2.277a2 2 0 0 1 1.788 0z",
                )],
            ),
            child("path", &[("d", "M15 5.764v15")]),
            child("path", &[("d", "M9 3.236v15")]),
        ],
    },
    IconDef {
        names: &["loader", "refresh"],
        children: &[
            child(
                "path",
                &[("d", "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74")],
            ),
            child("path", &[("d", "M21 3v6h-6")]),
        ],
    },
    IconDef {
        names: &["folder-open"],
        children: &[child(
            "path",
            &[(
                "d",
                "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2",
            )],
        )],
    },
    IconDef {
        names: &["square-terminal"],
        children: &[
            child(
                "rect",
                &[
                    ("width", "18"),
                    ("height", "18"),
                    ("x", "3"),
                    ("y", "3"),
                    ("rx", "2"),
                ],
            ),
            child("path", &[("d", "m7 11 2-2-2-2")]),
            child("path", &[("d", "M11 13h4")]),
        ],
    },
    IconDef {
        names: &["globe"],
        children: &[
            child("circle", &[("cx", "12"), ("cy", "12"), ("r", "10")]),
            child(
                "path",
                &[("d", "M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20")],
            ),
            child("path", &[("d", "M2 12h20")]),
        ],
    },
    IconDef {
        names: &["rotate-cw"],
        children: &[
            child(
                "path",
                &[("d", "M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8")],
            ),
            child("path", &[("d", "M21 3v5h-5")]),
        ],
    },
];

fn lookup(name: &str) -> Option<&'static [SvgChild]> {
    ICON_TABLE
        .iter()
        .find(|icon| icon.names.contains(&name))
        .map(|icon| icon.children)
}

fn render_child(out: &mut String, child: &SvgChild) {
    out.push('<');
    out.push_str(child.tag);
    out.push(' ');
    for (key, value) in child.attrs {
        let _ = write!(out, "{key}=\"{value}\" ");
    }
    out.push_str("/>");
}

/// Map an icon name to a complete SVG HTML string.
#[must_use]
pub fn icon_svg_html(name: &str) -> Option<String> {
    let children = lookup(name)?;
    let mut body = String::new();
    for child in children {
        render_child(&mut body, child);
    }
    Some(format!("<svg {SVG_ATTRS}>{body}</svg>"))
}

/// Map an icon name to structured SVG child elements for rsx! construction.
#[must_use]
pub fn icon_children(name: &str) -> Option<Vec<SvgChild>> {
    lookup(name).map(<[SvgChild]>::to_vec)
}

/// Resolve a kebab-case icon name (`"chevron-down"`, `"square-terminal"`)
/// into the `PascalCase` `gpui_component::IconName` variant ident
/// (`ChevronDown`, `SquareTerminal`). Falls back to `Info` for unknown
/// names, mirroring the runtime `resolve_icon` behavior of the adapters.
#[must_use]
pub fn icon_variant_ident(name: &str) -> proc_macro2::Ident {
    fn is_alias(name: &str) -> bool {
        ICON_TABLE.iter().any(|icon| icon.names.contains(&name))
    }
    let normalized = match name {
        // The lucide names differ slightly from the gpui-component asset
        // names; map the few used aliases here.
        "trash" => "delete",
        "refresh" => "rotate-cw",
        _ => name,
    };
    if !is_alias(normalized) {
        return proc_macro2::Ident::new("Info", proc_macro2::Span::call_site());
    }
    let pascal: String = normalized
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string())
                .unwrap_or_default()
                + chars.as_str()
        })
        .collect();
    proc_macro2::Ident::new(&pascal, proc_macro2::Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_icons_resolve() {
        assert!(icon_children("calendar").is_some());
        assert!(icon_children("x").is_some(), "aliases resolve");
        assert!(icon_children("close").is_some());
        assert!(icon_svg_html("settings").is_some());
    }

    #[test]
    fn unknown_icons_return_none() {
        assert!(icon_children("no-such-icon").is_none());
        assert!(icon_svg_html("").is_none());
    }

    #[test]
    fn svg_html_is_wellformed() {
        let html = icon_svg_html("check").expect("check icon");
        assert!(html.starts_with("<svg "));
        assert!(html.ends_with("</svg>"));
        assert!(html.contains("<path d=\"M20 6 9 17l-5-5\""));
    }

    #[test]
    fn every_icon_renders() {
        for icon in ICON_TABLE {
            for name in icon.names {
                assert!(icon_svg_html(name).is_some(), "icon {name} must render");
                assert!(!icon.children.is_empty(), "icon {name} has children");
            }
        }
    }
}
