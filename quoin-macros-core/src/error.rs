use proc_macro2::Span;
use thiserror::Error;

/// Centralized error type for Quoin macro parsing.
#[derive(Error, Debug)]
pub enum QuoinError {
    /// A required macro attribute was not supplied.
    #[error("missing required attribute `{attr}`")]
    MissingAttribute {
        /// Name of the missing attribute.
        attr: &'static str,
    },

    /// A `state` field was declared without an initial value.
    #[error("state field `{name}` requires a default value")]
    StateRequiresDefault {
        /// Name of the offending field.
        name: String,
    },

    /// An identifier appeared where none was expected.
    #[error("unexpected identifier `{found}`")]
    UnexpectedIdent {
        /// The unexpected identifier.
        found: String,
    },

    /// An argument name was not followed by `:`.
    #[error("expected `:` after argument name `{key}`")]
    MissingColon {
        /// The argument name.
        key: String,
    },

    /// The macro input had no `render { ... }` block.
    #[error("missing render block")]
    MissingRenderBlock,

    /// A Tailwind class could not be transpiled for the GPUI backend.
    #[error("unknown Tailwind class: `{class}`")]
    UnknownTailwindClass {
        /// The untranslatable class name.
        class: String,
    },
}

impl QuoinError {
    /// Helper to convert into a `syn::Error` at a specific span.
    #[must_use]
    pub fn to_syn_error_at(&self, span: Span) -> syn::Error {
        syn::Error::new(span, self.to_string())
    }

    /// Helper to convert into a `syn::Error` spanning a specific token.
    #[must_use]
    pub fn to_syn_error_spanning<T: quote::ToTokens>(&self, tokens: &T) -> syn::Error {
        syn::Error::new_spanned(tokens, self.to_string())
    }
}
