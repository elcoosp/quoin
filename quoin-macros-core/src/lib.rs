//! Core logic for the `quoin` procedural macros.
//!
//! This crate separates the macro parsing and emission logic from the
//! `proc_macro` crate boundary, allowing types and traits to be exported
//! for downstream consumption and testing.

pub mod custom_element;
pub mod effect;
pub mod error;
pub mod parse;
pub mod render_ast;

pub mod emit;
pub mod render_ast_diag;
pub mod run_app;
pub mod transpile;
