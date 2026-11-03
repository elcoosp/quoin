//! Typed style IR shared by every quoin backend.
//!
//! `quoin_render!(div(class: "flex flex-col gap-2 hover:bg-primary"))` parses
//! the class string **at compile time** into this crate's typed vocabulary:
//! a list of [`Decl`] values for the base state, a list for the `hover:`
//! state, and a list of classes that could not be recognized ([`ParsedClasses::unknown`]).
//!
//! One parse, many lowerings. A proc-macro backend (GPUI, Floem, Xilem, …)
//! walks the `Decl` list and emits framework-native builder calls. Because
//! parsing happens once, the emitted code contains no runtime class
//! parsing and no string lookups — the class string is gone by the time the
//! user's crate compiles.
//!
//! # Vocabulary
//!
//! [`Decl`] models the subset of Tailwind that quoin supports: flexbox
//! direction/wrap/alignment, gap, padding, margin, width/height (as
//! [`Len`]), background and border colors (as [`Col`], including theme
//! tokens), border widths, corner radii, opacity, cursor, position offsets,
//! overflow, shadows, and the text properties (color, size, weight,
//! alignment, family).
//!
//! # Unknown classes are a compile error
//!
//! [`parse_tailwind`] never silently drops a class it does not know:
//! unrecognized classes land in [`ParsedClasses::unknown`], and every
//! backend is required to turn them into a `compile_error!` with a
//! did-you-mean suggestion (see [`suggest_class`]).
//!
//! # Theme tokens
//!
//! `bg-primary`, `text-muted-foreground`, … parse into
//! [`Col::Token`]. Backends resolve tokens their own way: the GPUI backend
//! bakes in the default shadcn palette at expansion time (matching its
//! pre-existing behavior), while the Floem and Xilem backends emit calls to
//! their adapter's `theme::primary()`-style functions, which read lock-free
//! atomics so a palette can be swapped at runtime.

use core::cmp::Ordering;

pub mod decl;
pub mod parse;

pub use decl::{
    BorderSide, Col, Corner, Cursor, Decl, Display, FlexDir, HexLit, Items, Justify, Len, Overflow,
    Position, Shadow, TextAlign, TextScale, Token, Weight,
};
pub use parse::{KNOWN_CLASSES, ParsedClasses, parse_tailwind};

/// Returns the closest known class to `input` within edit distance 2, if any.
///
/// Used by every backend's unknown-class diagnostic. Plain Levenshtein with
/// an early cutoff; the candidate list is small enough that the quadratic
/// scan is irrelevant next to the rest of the proc-macro run.
#[must_use]
pub fn suggest_class(input: &str) -> Option<&'static str> {
    let mut best: Option<(&'static str, usize)> = None;
    for candidate in KNOWN_CLASSES {
        let distance = levenshtein_bounded(input, candidate, 2);
        if distance > 2 {
            continue;
        }
        let better = match best {
            None => true,
            Some((best_name, best_distance)) => match distance.cmp(&best_distance) {
                Ordering::Less => true,
                Ordering::Equal => candidate.len() < best_name.len(),
                Ordering::Greater => false,
            },
        };
        if better {
            best = Some((candidate, distance));
        }
    }
    best.map(|(name, _)| name)
}

/// Bounded Levenshtein distance: returns `max + 1` once the distance is known
/// to exceed `max`, so near-duplicate scans stay cheap.
fn levenshtein_bounded(a: &str, b: &str, max: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return max + 1;
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        let mut row_min = current[0];
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current[j + 1] = (previous[j + 1] + 1).min(current[j] + 1).min(substitution);
            row_min = row_min.min(current[j + 1]);
        }
        if row_min > max {
            return max + 1;
        }
        core::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_finds_transpositions() {
        assert_eq!(suggest_class("flec-row"), Some("flex-row"));
        assert_eq!(suggest_class("gap2"), Some("gap-2"));
        assert_eq!(suggest_class("flex_col"), Some("flex-col"));
    }

    #[test]
    fn suggest_rejects_distant_input() {
        assert_eq!(suggest_class("zzzzzzzz"), None);
    }

    #[test]
    fn suggest_prefers_shorter_candidates_on_tie() {
        // "border" and "border-b" are both one edit from "borderb"; the
        // shorter candidate wins.
        assert_eq!(suggest_class("borderb"), Some("border"));
    }
}
