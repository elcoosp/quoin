//! The machine-checked rendering-parity table.
//!
//! One row per element on the v1 path, one status per dynamic backend
//! (Floem, Xilem). `parity_matrix.rs` cross-checks this table against the
//! real emitters — a row claiming [`ParityLevel::Native`] must expand
//! without a diagnostic, and any `compile_error!` an emitter produces for a
//! known element must correspond to a [`ParityLevel::Unsupported`] row here.
//! `docs/BACKEND-PARITY.md` renders the same table (plus the GPUI / Leptos /
//! Dioxus reference columns) and is checked to mention every row.
//!
//! GPUI / Leptos / Dioxus are deliberately **not** in the const: they are
//! the pre-existing reference backends whose element handling predates the
//! parity plan (GPUI notably lowers unknown tags to plain styled divs), so
//! their "status" is not a contract this table can machine-verify.

/// The dynamic backends the table covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Back {
    /// The Floem backend (`quoin-floem`).
    Floem,
    /// The Xilem backend (`quoin-xilem`).
    Xilem,
}

/// How a backend lowers one element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityLevel {
    /// Lowers to a native view of that framework; semantics match the
    /// shared contract.
    Native,
    /// Lowers to a native view with a documented, accepted difference.
    Partial(&'static str),
    /// Emits a named `compile_error!` diagnostic. The reason is the
    /// permanent contract — not "lands with a later push".
    Unsupported(&'static str),
}

/// One element's status on both dynamic backends.
#[derive(Debug, Clone, Copy)]
pub struct ElementParity {
    /// The element name as written in `quoin_render!`.
    pub element: &'static str,
    /// Floem status.
    pub floem: ParityLevel,
    /// Xilem status.
    pub xilem: ParityLevel,
}

/// The v1 parity matrix. Keep in sync with the emitters and
/// `docs/BACKEND-PARITY.md` — the parity-matrix test enforces both.
pub const BACKEND_PARITY: &[ElementParity] = &[
    ElementParity {
        element: "button",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "input",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "checkbox",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "switch",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "slider",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "progress",
        floem: ParityLevel::Partial("rendered as a disabled slider bar"),
        xilem: ParityLevel::Partial("rendered as a disabled slider bar"),
    },
    ElementParity {
        element: "badge",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "skeleton",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "skeleton_text",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "skeleton_avatar",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "separator",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "scroll_area",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "virtual_list",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "tabs",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Native,
    },
    ElementParity {
        element: "data_table",
        floem: ParityLevel::Partial(
            "rows rebuild through a dyn_container; no row-keyed diffing (the element contract has no row key)",
        ),
        xilem: ParityLevel::Partial("rows rebuild as a fresh Vec sequence each render"),
    },
    ElementParity {
        element: "dropdown_menu",
        floem: ParityLevel::Partial(
            "inline item column (matching the GPUI ghost-menu fallback); no portal overlay",
        ),
        xilem: ParityLevel::Partial("inline item column; no portal overlay"),
    },
    ElementParity {
        element: "clipboard_button",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Unsupported("xilem/masonry exposes no clipboard API on the v1 branch"),
    },
    ElementParity {
        element: "tooltip",
        floem: ParityLevel::Native,
        xilem: ParityLevel::Unsupported(
            "masonry has no tooltip widget; wrapping in a hint container changes semantics, so v1 rejects it",
        ),
    },
    ElementParity {
        element: "icon",
        floem: ParityLevel::Unsupported(
            "the icon-name table is gpui-component-specific; pass an explicit `svg`/styled label instead",
        ),
        xilem: ParityLevel::Unsupported(
            "the icon-name table is gpui-component-specific; pass an explicit `svg`/styled label instead",
        ),
    },
    ElementParity {
        element: "img",
        floem: ParityLevel::Unsupported(
            "not in the v1 cross-backend element contract (only the web backends pass `img` through as HTML)",
        ),
        xilem: ParityLevel::Unsupported(
            "not in the v1 cross-backend element contract (only the web backends pass `img` through as HTML)",
        ),
    },
    ElementParity {
        element: "svg",
        floem: ParityLevel::Unsupported(
            "not in the v1 cross-backend element contract; use a styled label or a backend-specific view",
        ),
        xilem: ParityLevel::Unsupported(
            "not in the v1 cross-backend element contract; use a styled label or a backend-specific view",
        ),
    },
    ElementParity {
        element: "rich_text",
        floem: ParityLevel::Unsupported(
            "not on the v1 path; the shared text contract is single-style text",
        ),
        xilem: ParityLevel::Unsupported(
            "not on the v1 path; the shared text contract is single-style text",
        ),
    },
    ElementParity {
        element: "styled_text",
        floem: ParityLevel::Unsupported(
            "not on the v1 path; the shared text contract is single-style text",
        ),
        xilem: ParityLevel::Unsupported(
            "not on the v1 path; the shared text contract is single-style text",
        ),
    },
    ElementParity {
        element: "radio",
        floem: ParityLevel::Unsupported(
            "the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox`",
        ),
        xilem: ParityLevel::Unsupported(
            "the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox`",
        ),
    },
    ElementParity {
        element: "radio_group",
        floem: ParityLevel::Unsupported(
            "the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox`",
        ),
        xilem: ParityLevel::Unsupported(
            "the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox`",
        ),
    },
];

/// Looks up one element's row.
#[must_use]
pub fn row(element: &str) -> Option<&'static ElementParity> {
    BACKEND_PARITY.iter().find(|r| r.element == element)
}
