//! Typed events shared by every quoin backend (`quoin::ev`).
//!
//! Handlers receive small value types instead of framework-native event
//! objects, so the same handler source compiles on all five backends. The
//! native event stays reachable: each adapter converts from its own event
//! type through `From` impls that are `#[inline(always)]` in generated code.
//!
//! # Payload contract
//!
//! - Zero-parameter handlers (`|| …`) remain valid on every backend; the
//!   adapter calls them with no payload at all.
//! - One-parameter handlers receive one of these structs. During the M4
//!   transition `()` still coerces for zero-arg handlers, which keeps GPUI
//!   source compiling unchanged.

/// A primary click / activation. Floem's `action` decorator and Xilem's
/// button callbacks fire this; adapters with richer pointer data widen the
/// struct in later milestones without changing handler arity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Click {}

impl Click {
    /// The event every backend passes for a plain activation.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

/// A text edit from an input/textarea element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    /// The full, current text of the field after the edit.
    pub value: std::sync::Arc<str>,
}

impl Input {
    /// Builds an `Input` event from the field's current text.
    #[must_use]
    pub fn new(value: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// A key press. Populated by the M5 `keys { .. }` lowering; adapters without
/// key routing (Xilem until M5) emit a `compile_error!` at the element site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    /// The physical key code, e.g. `"Escape"`, `"KeyA"` (W3C `code` names).
    pub code: &'static str,
    /// Whether a platform modifier (Ctrl/Cmd) was held.
    pub ctrl: bool,
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Alt/Option was held.
    pub alt: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_event_carries_value() {
        let ev = Input::new("hello");
        assert_eq!(ev.value.as_ref(), "hello");
    }

    #[test]
    fn click_is_a_zero_sized_value() {
        assert_eq!(Click::new(), Click {});
        assert_eq!(size_of::<Click>(), 0);
    }
}
