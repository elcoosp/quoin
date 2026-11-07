//! gpui-rsx-backed class lowering for the GPUI backend (feature
//! `gpui-rsx-classes`, migration report §8 Phase 1 + Phase 2).
//!
//! # Policy — what changes and what cannot change
//!
//! Classes quoin's parser already knows keep their existing, byte-identical
//! lowering ([`super::tailwind::decl_to_gpui_tokens`]). The engine only
//! decides classes that quoin *rejects* today, which is exactly where the
//! migration is meant to gain ground without risking a regression:
//!
//! | quoin parse | gpui-rsx outcome | action |
//! |-------------|------------------|--------|
//! | known decl (incl. theme tokens) | — (not consulted) | existing lowering, byte-identical |
//! | unknown | `Supported` | gpui-rsx's tokens — **new vocabulary** (`text-blue-400`, `w-1/2`, arbitrary values, negative utilities) |
//! | unknown | `Supported` + `focus:`/`active:`/`group-*:` variant | same, plus the element is made stateful (`.id(..)`) so the refinement compiles |
//! | unknown | `Error` / `Ignored` | `compile_error!` carrying gpui-rsx's message and quoin's did-you-mean |
//! | unknown | `PassThrough` | unreachable in strict mode; treated as unknown |
//!
//! Phase 5 of the migration plan ("flip the default") is the point where the
//! priority order reverses and gpui-rsx's engine takes over the known-class
//! lowering as well.
//!
//! # Theme extension via `rsx.toml` (Phase 2)
//!
//! When the crate being compiled carries an `rsx.toml` next to its
//! `Cargo.toml` (the same convention gpui-rsx uses), the file is loaded at
//! expansion time and handed to the engine, so custom tokens
//! (`bg-brand = "#ff0000"`) resolve like any other class. The file is also
//! tracked with an `include_bytes!` rebuild marker so editing it recompiles
//! the invoking crate. quoin's own 17 theme tokens are *not* routed through
//! the theme — they keep their baked shadcn-dark hex output.

use std::sync::Arc;

use proc_macro2::TokenStream;
use quote::quote;

use gpui_rsx_core::theme::rebuild_tracker;
use gpui_rsx_core::{ClassMode, ClassOutcome, SpacingScale, Theme, analyze_class};
use quoin_style::{Decl, suggest_class};

use super::tailwind::{TranspiledStyles, decl_to_gpui_tokens};

/// The engine's class mode: strict (unknown classes are compile errors,
/// preserving quoin's fail-fast contract) with quoin's 4 px-per-unit
/// Tailwind spacing scale, plus whatever theme the invoking crate declares.
fn engine_mode(theme: Option<Arc<Theme>>) -> ClassMode {
    ClassMode::STRICT
        .with_scale(SpacingScale::Tailwind)
        .with_theme(theme)
}

/// Loads the invoking crate's `rsx.toml`, if any.
///
/// `CARGO_MANIFEST_DIR` is set for the rustc process compiling a package, so
/// a proc macro observes the *invoking* crate's manifest dir — the same
/// convention gpui-rsx's `rsx!` uses. A broken file is a compile error
/// (matching gpui-rsx's "broken configs are compile errors" behavior); a
/// missing file is simply no theme.
fn user_theme() -> Result<Option<(Arc<Theme>, TokenStream)>, String> {
    let Some(dir) = std::env::var_os("CARGO_MANIFEST_DIR") else {
        return Ok(None);
    };
    match Theme::load_from_dir(std::path::Path::new(&dir)) {
        Ok(Some(theme)) => {
            let tracker = rebuild_tracker(&theme).unwrap_or_default();
            Ok(Some((theme, tracker)))
        }
        Ok(None) => Ok(None),
        Err(message) => Err(message),
    }
}

/// Builds one `compile_error!` for an unknown class, matching the default
/// engine's diagnostic wording, with a did-you-mean suggestion when
/// available.
fn unknown_class_error(class: &str, suggestion: Option<&str>) -> TokenStream {
    let message = match suggestion {
        Some(sug) => {
            format!("unknown Tailwind class `{class}` (in `class: \"…\"`). Did you mean `{sug}`?")
        }
        None => format!(
            "unknown Tailwind class `{class}` (in `class: \"…\"`). See `quoin_style::KNOWN_CLASSES` for the supported vocabulary."
        ),
    };
    quote! { ::core::compile_error!(#message); }
}

/// Builds a `compile_error!` carrying an engine rejection message (strict
/// unknown, invalid value, unsupported variant, …) plus a did-you-mean when
/// quoin's suggestion finds a close neighbor.
fn engine_error(class: &str, message: &str) -> TokenStream {
    let suggestion = suggest_class(class);
    let message = match suggestion {
        Some(sug) => format!("{message} Did you mean `{sug}`?"),
        None => message.to_owned(),
    };
    quote! { ::core::compile_error!(#message); }
}

/// Whether a raw class string is one of the scroll overflow classes, used to
/// keep the stateful-`.id()` injection working for overflow classes that
/// quoin's own vocabulary does not know.
fn raw_is_scroll_class(class: &str) -> bool {
    class.starts_with("overflow-") && (class.contains("scroll") || class.contains("auto"))
}

/// Qualifies the bare GPUI value constructors gpui-rsx's engine emits
/// (`px(…)`, `rems(…)`, `relative(…)`, `rgb(…)`, `rgba(…)` and
/// `FontWeight::…`) with the fully-qualified `::gpui::` path, matching how
/// the default engine lowers values (`gpui::px(16.0)`).
///
/// quoin's macro output must be self-contained: gpui-rsx's own `rsx!` relies
/// on a `use gpui::*` at the call site, which quoin's `quoin_render!` cannot
/// assume. The traversal recurses into groups and preserves every span.
fn qualify_gpui_values(tokens: TokenStream) -> TokenStream {
    use proc_macro2::{Delimiter, Group, Spacing, TokenTree};

    const CONSTRUCTORS: [&str; 5] = ["px", "rems", "relative", "rgb", "rgba"];

    fn global_path_prefix() -> Vec<TokenTree> {
        // `::gpui::` — the leading root plus the member separator.
        let colon1 = proc_macro2::Punct::new(':', Spacing::Joint);
        let colon2 = proc_macro2::Punct::new(':', Spacing::Alone);
        let gpui = proc_macro2::Ident::new("gpui", proc_macro2::Span::call_site());
        let colon3 = proc_macro2::Punct::new(':', Spacing::Joint);
        let colon4 = proc_macro2::Punct::new(':', Spacing::Alone);
        vec![
            TokenTree::Punct(colon1),
            TokenTree::Punct(colon2),
            TokenTree::Ident(gpui),
            TokenTree::Punct(colon3),
            TokenTree::Punct(colon4),
        ]
    }

    fn rewrite(stream: TokenStream) -> Vec<TokenTree> {
        let mut out = Vec::new();
        let mut iter = stream.into_iter().peekable();
        while let Some(tt) = iter.next() {
            match tt {
                TokenTree::Group(group) => {
                    let inner = rewrite(group.stream());
                    let mut rebuilt = Group::new(group.delimiter(), inner.into_iter().collect());
                    rebuilt.set_span(group.span());
                    out.push(TokenTree::Group(rebuilt));
                }
                TokenTree::Ident(ident) => {
                    let name = ident.to_string();
                    let is_constructor_call = CONSTRUCTORS.contains(&name.as_str())
                        && iter
                            .peek()
                            .is_some_and(|next| {
                                matches!(next, TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis)
                            });
                    let is_font_weight = name == "FontWeight";
                    if is_constructor_call || is_font_weight {
                        out.extend(global_path_prefix());
                    }
                    out.push(TokenTree::Ident(ident));
                }
                other => out.push(other),
            }
        }
        out
    }

    rewrite(tokens).into_iter().collect()
}

/// One GPUI token stream per resolved class, routed into the normal/hover
/// lists by the variant gpui-rsx reports.
fn route_resolved(
    analysis: gpui_rsx_core::ClassAnalysis,
    class: &str,
    normal: &mut Vec<TokenStream>,
    hover: &mut Vec<TokenStream>,
    errors: &mut Vec<TokenStream>,
    stateful: &mut bool,
) {
    let ClassOutcome::Supported(tokens) = analysis.outcome else {
        // The caller already filtered non-Supported outcomes; this arm only
        // exists because `ClassOutcome` is `#[non_exhaustive]`.
        errors.push(unknown_class_error(class, None));
        return;
    };
    let tokens = qualify_gpui_values(tokens);
    match analysis.variant {
        None => normal.push(tokens),
        // `hover:` keeps the default engine's shape: a bare token fragment
        // routed into the `hover` list, wrapped once by
        // `apply_normal_and_hover` as `.hover(|__s| __s …)`.
        Some(variant) if variant.method == "hover" && variant.group.is_none() => {
            hover.push(tokens);
        }
        Some(variant) => {
            // focus / focus-visible / active / group-hover / group-active:
            // `StatefulInteractiveElement` refinements — the element must be
            // stateful before they chain. The method names come from
            // gpui-rsx's generated variant table for quoin's pinned GPUI
            // snapshot. These tokens are already fully-wrapped refinement
            // calls, so they ride in the normal list.
            *stateful = true;
            let method = proc_macro2::Ident::new(variant.method, proc_macro2::Span::call_site());
            let wrapped = if let Some(group) = variant.group {
                quote! { .#method(#group, |__s| __s #tokens) }
            } else {
                quote! { .#method(|__s| __s #tokens) }
            };
            normal.push(wrapped);
        }
    }
}

/// The `gpui-rsx-classes` engine: transpiles one `class=".."` string.
///
/// See the module docs for the decision policy.
pub(crate) fn transpile_class(class_str: &str) -> TranspiledStyles {
    let mut normal = Vec::new();
    let mut hover = Vec::new();
    let mut errors = Vec::new();
    let mut scroll = false;
    let mut stateful = false;

    let theme = match user_theme() {
        Ok(pair) => pair,
        Err(message) => {
            let error = quote! { ::core::compile_error!(#message); };
            return TranspiledStyles {
                normal,
                hover,
                errors: vec![error],
                scroll,
                stateful,
                dynamic: None,
                rebuild: None,
            };
        }
    };
    let rebuild = theme.as_ref().map(|(_, tracker)| tracker.clone());
    let mode = engine_mode(theme.map(|(theme, _)| theme));

    for class in class_str.split_ascii_whitespace() {
        let parsed = quoin_style::parse_tailwind(class);
        if parsed.unknown.is_empty() {
            // quoin knows the class — byte-identical output, unchanged.
            for decl in &parsed.base {
                normal.push(decl_to_gpui_tokens(decl));
            }
            for decl in &parsed.hover {
                hover.push(decl_to_gpui_tokens(decl));
            }
            if parsed
                .base
                .iter()
                .chain(&parsed.hover)
                .any(|decl| matches!(decl, Decl::Overflow(o) if o.needs_stateful_scroll()))
            {
                scroll = true;
            }
            continue;
        }

        // quoin rejects the class: the gpui-rsx engine decides.
        let analysis = analyze_class(class, mode.clone());
        match analysis.outcome {
            ClassOutcome::Supported(_) => {
                route_resolved(
                    analysis,
                    class,
                    &mut normal,
                    &mut hover,
                    &mut errors,
                    &mut stateful,
                );
                if raw_is_scroll_class(class) {
                    scroll = true;
                }
            }
            ClassOutcome::PassThrough { suggestion, .. } => {
                // Strict mode makes PassThrough unreachable for unknown
                // classes; treat any future occurrence as unknown.
                errors.push(unknown_class_error(class, suggestion.as_deref()));
            }
            ClassOutcome::Ignored { message } | ClassOutcome::Error { message } => {
                errors.push(engine_error(class, &message));
            }
            // `#[non_exhaustive]` future outcomes are compile errors too
            // (strict engine, fail-fast contract).
            _ => {
                errors.push(unknown_class_error(class, None));
            }
        }
    }

    TranspiledStyles {
        normal,
        hover,
        errors,
        scroll,
        stateful,
        dynamic: None,
        rebuild,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────
//
// These pin the decision policy from the module docs: quoin-known classes
// keep the default engine's lowering, quoin-unknown classes resolve through
// gpui-rsx (new vocabulary), unknown-to-both classes are compile errors with
// suggestions, variants route correctly and mark the element stateful, and
// dynamic classes carry the class expression.
#[cfg(test)]
mod tests {
    use super::{TranspiledStyles, transpile_class};
    use quoin_style::suggest_class;

    fn norm(ts: &proc_macro2::TokenStream) -> String {
        ts.to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }

    fn token_string(tokens: &[proc_macro2::TokenStream]) -> String {
        tokens.iter().map(norm).collect::<Vec<_>>().join(";")
    }

    // ── quoin-known classes: byte-identical to the default engine ───────────
    #[test]
    fn known_classes_keep_the_default_lowering() {
        let cases = [
            ("flex", ".flex()"),
            ("gap-4", ".gap(gpui::px(16.0))"),
            ("p-4", ".p(gpui::px(16.0))"),
            ("bg-gray-900", ".bg(gpui::rgb(0x111827))"),
            ("text-white", ".text_color(gpui::white())"),
            ("text-2xl", ".text_2xl()"),
            ("font-bold", ".font_weight(gpui::FontWeight::BOLD)"),
            ("rounded-md", ".rounded(gpui::px(6.0))"),
            ("border-b", ".border_b_1()"),
            ("w-full", ".w_full()"),
            ("opacity-50", ".opacity(0.5)"),
            ("overflow-y-scroll", ".overflow_y_scroll()"),
            ("whitespace-nowrap", ".whitespace_nowrap()"),
        ];
        for (class, expected) in cases {
            let styles = transpile_class(class);
            assert!(
                styles.errors.is_empty(),
                "`{class}` unexpectedly errored: {}",
                token_string(&styles.errors)
            );
            assert_eq!(
                token_string(&styles.normal),
                expected,
                "known class `{class}` drifted from the default engine"
            );
        }
    }

    #[test]
    fn hover_classes_split_like_the_default_engine() {
        let styles = transpile_class("hover:bg-gray-800");
        assert!(styles.errors.is_empty());
        assert!(styles.normal.is_empty());
        assert_eq!(token_string(&styles.hover), ".bg(gpui::rgb(0x1f2937))");
    }

    #[test]
    fn theme_tokens_keep_their_baked_hexes() {
        let cases = [
            ("bg-primary", ".bg(gpui::rgb(6514417))"),
            ("bg-secondary", ".bg(gpui::rgb(1976635))"),
            ("text-secondary", ".text_color(gpui::rgb(9741240))"),
            ("text-muted-foreground", ".text_color(gpui::rgb(9741240))"),
            ("bg-destructive", ".bg(gpui::rgb(4524554))"),
            ("bg-card", ".bg(gpui::rgb(1976635))"),
            ("text-popover", ".text_color(gpui::rgb(1976635))"),
        ];
        for (class, expected) in cases {
            let styles = transpile_class(class);
            assert!(
                styles.errors.is_empty(),
                "`{class}` unexpectedly errored: {}",
                token_string(&styles.errors)
            );
            assert_eq!(token_string(&styles.normal), expected);
        }
    }

    // ── New vocabulary: classes quoin rejects today ─────────────────────────
    #[test]
    fn gpui_rsx_resolves_new_vocabulary() {
        // text-blue-400: 0x60a5fa = 6333946 (migration report §4.2 evidence).
        let styles = transpile_class("text-blue-400");
        assert!(styles.errors.is_empty());
        assert_eq!(
            token_string(&styles.normal),
            ".text_color(::gpui::rgb(6333946u32))"
        );

        let styles = transpile_class("w-1/2 border-t-2");
        assert!(
            styles.errors.is_empty(),
            "fraction/width classes must resolve: {}",
            token_string(&styles.errors)
        );
        assert!(!styles.normal.is_empty());

        let styles = transpile_class("w-[280px]");
        assert!(styles.errors.is_empty());
        assert!(token_string(&styles.normal).contains("280"));
    }

    #[test]
    fn negative_utilities_are_rejected_in_strict_mode() {
        // gpui-rsx 0.10.2 rejects negative utilities in strict mode with its
        // own guidance ("Did you mean `mt-2`?"). Strict is the engine's
        // default here, preserving quoin's fail-fast contract; the class is
        // not silently accepted.
        let styles = transpile_class("-mt-2");
        assert!(!styles.errors.is_empty(), "strict mode must reject -mt-2");
        let s = norm(&styles.errors[0]);
        assert!(
            s.contains("strictmode"),
            "expected the strict-mode guidance, got `{s}`"
        );
    }

    #[test]
    fn non_hover_variants_mark_the_element_stateful() {
        let styles = transpile_class("focus:border-blue-500");
        assert!(styles.errors.is_empty());
        assert!(styles.stateful, "focus: requires a stateful element");
        // The refinement rides in the normal list, already wrapped.
        let s = norm(&styles.normal[0]);
        assert!(
            s.starts_with(".focus("),
            "expected a focus refinement, got `{s}`"
        );
        assert!(s.contains("border_color"), "refinement body lost: `{s}`");
    }

    #[test]
    fn grouped_variants_carry_their_group() {
        let styles = transpile_class("group-hover/menu:bg-red-500");
        assert!(styles.errors.is_empty());
        assert!(styles.stateful);
        let s = norm(&styles.normal[0]);
        assert!(s.contains("group_hover"), "expected group_hover, got `{s}`");
        assert!(s.contains("\"menu\""), "group name lost: `{s}`");
    }

    #[test]
    fn hover_via_new_vocabulary_uses_the_hover_list() {
        // hover:text-blue-400 is unknown to quoin (blue-400 is not in its
        // text palette) but known to the engine — it must route like any
        // hover class.
        let styles = transpile_class("hover:text-blue-400");
        assert!(styles.errors.is_empty());
        assert!(styles.normal.is_empty());
        let s = norm(&styles.hover[0]);
        assert!(s.contains("text_color"), "hover body lost: `{s}`");
    }

    // ── Unknown to both engines: compile error with suggestion ────────────
    #[test]
    fn classes_unknown_to_both_engines_are_compile_errors() {
        let styles = transpile_class("totally-not-a-class");
        assert!(!styles.errors.is_empty(), "fail-fast contract must hold");
        let s = norm(&styles.errors[0]);
        assert!(
            s.contains("compile_error!"),
            "expected compile_error, got `{s}`"
        );
        assert!(
            s.contains("totally-not-a-class"),
            "error must name the class"
        );
    }

    #[test]
    fn unsupported_responsive_variants_are_compile_errors() {
        // `dark:` is rejected by gpui-rsx with an explanatory message.
        let styles = transpile_class("dark:bg-black");
        assert!(!styles.errors.is_empty());
        let s = norm(&styles.errors[0]);
        assert!(
            s.contains("not supported") || s.contains("compile_error!"),
            "expected an explanatory error, got `{s}`"
        );
    }

    #[test]
    fn error_messages_keep_the_did_you_mean_suggestion() {
        // A close neighbor of a known class: quoin's suggestion machinery
        // fires on top of the engine's rejection message.
        let class = "flx";
        let suggestion = suggest_class(class);
        let styles = transpile_class(class);
        assert!(!styles.errors.is_empty());
        let s = norm(&styles.errors[0]);
        if suggestion.is_some() {
            assert!(s.contains("Didyoumean"), "suggestion lost: `{s}`");
        }
    }

    // ── Mixed strings: both engines in one class argument ──────────────────
    #[test]
    fn mixed_strings_dispatch_per_class() {
        let styles = transpile_class("flex flex-col gap-4 text-blue-400 hover:text-blue-300");
        assert!(
            styles.errors.is_empty(),
            "mixed string errored: {}",
            token_string(&styles.errors)
        );
        let normal = token_string(&styles.normal);
        assert!(
            normal.contains(".flex()"),
            "quoin-known class missing: `{normal}`"
        );
        assert!(
            normal.contains("text_color"),
            "engine-resolved class missing: `{normal}`"
        );
        assert!(!styles.hover.is_empty(), "engine-resolved hover missing");
    }

    #[test]
    fn scroll_detection_covers_both_engines() {
        // quoin-known scroll class → decl-based detection.
        assert!(transpile_class("overflow-y-scroll").needs_stateful());
        // Mixed with a new-vocabulary class the flag survives.
        assert!(transpile_class("flex overflow-y-scroll text-blue-400").needs_stateful());
        assert!(!transpile_class("flex text-blue-400").needs_stateful());
    }

    // ── Dynamic classes carry the expression ───────────────────────────────
    #[test]
    fn dynamic_class_constructor_carries_the_expression() {
        let expr: syn::Expr = syn::parse_str("accent.clone()").expect("parses");
        let styles = TranspiledStyles::dynamic_class(&expr);
        assert!(styles.dynamic.is_some());
        assert_eq!(
            norm(&styles.dynamic.as_ref().expect("checked")),
            "accent.clone()"
        );
        assert!(styles.normal.is_empty() && styles.hover.is_empty() && styles.errors.is_empty());
        assert!(!styles.needs_stateful());
    }

    #[test]
    fn dynamic_classes_reach_try_transpile_class() {
        // End-to-end through the dispatch: a non-literal class expression
        // produces the dynamic styles rather than `None` (the silent drop).
        let expr: syn::Expr = syn::parse_str("some_signal.get()").expect("parses");
        let styles = super::super::super::emit::render_gpui::try_transpile_class(&expr)
            .expect("dynamic classes must resolve under the flag");
        assert!(styles.dynamic.is_some());
    }

    // ── rsx.toml theme extension (Phase 2) ────────────────────────────────
    #[test]
    fn user_rsx_toml_extends_the_vocabulary() {
        // quoin-macros-core carries no rsx.toml, so custom tokens must fail
        // fast (compile errors). Crates that *do* ship an rsx.toml resolve
        // their tokens through the engine (end-to-end covered in
        // quoin-grsx-compiletest and gpui-rsx-core's own theme tests).
        let styles = transpile_class("bg-brand-turquoise");
        assert!(
            !styles.errors.is_empty(),
            "unknown token must fail fast without rsx.toml"
        );
        assert!(styles.rebuild.is_none(), "no rsx.toml, no rebuild tracking");
    }
}
