//! Core logic for the `quoin` procedural macros.
//!
//! This crate separates the macro parsing and emission logic from the
//! `proc_macro` crate boundary, allowing types and traits to be exported
//! for downstream consumption and testing.

/// Parsing and expansion for the `quoin_element!` custom-element macro.
pub mod custom_element;
/// Parsing for the `effect!` macro AST.
pub mod effect;
/// Shared error type for macro parsing failures.
pub mod error;
/// Parser for the `component!` macro AST (`ComponentAst`).
pub mod parse;
/// Declarative-UI AST shared by every framework backend.
pub mod render_ast;

/// Framework-specific code emission for the parsed ASTs.
pub mod emit;
/// Diagnostic checks (required args, unknown args, typo suggestions) over
/// the render AST.
pub mod render_ast_diag;
/// Parser for the `run_app!` bootstrap macro.
pub mod run_app;
/// Tailwind-class and inline-SVG transpilation helpers for the GPUI backend.
pub mod transpile;
