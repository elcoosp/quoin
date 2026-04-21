//! Convenience macros for reactive programming.
//!
//! This module provides lightweight macros that simplify common patterns when
//! working with reactive signals and closures.

/// Reads a signal's current value. Expands to `.get()`.
#[macro_export]
macro_rules! read {
    ($signal:expr) => {
        $signal.get()
    };
}

/// Creates a move closure, automatically cloning specified variables before capture.
#[macro_export]
macro_rules! action {
    ($($capture:ident),* => $body:expr) => {{
        $(let $capture = std::clone::Clone::clone(&$capture);)*
        move || $body
    }};
}
