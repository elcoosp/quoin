use proc_macro2::Span;
use thiserror::Error;

/// Centralized error type for Quoin macro parsing.
#[derive(Error, Debug)]
pub enum QuoinError {
    #[error("missing required attribute `{attr}`")]
    MissingAttribute { attr: &'static str },

    #[error("state field `{name}` requires a default value")]
    StateRequiresDefault { name: String },

    #[error("unexpected identifier `{found}`")]
    UnexpectedIdent { found: String },

    #[error("expected `:` after argument name `{key}`")]
    MissingColon { key: String },

    #[error("missing render block")]
    MissingRenderBlock,

    #[error("unknown Tailwind class: `{class}`")]
    UnknownTailwindClass { class: String },
}

impl QuoinError {
    /// Helper to convert into a `syn::Error` at a specific span.
    pub fn to_syn_error_at(&self, span: Span) -> syn::Error {
        syn::Error::new(span, self.to_string())
    }

    /// Helper to convert into a `syn::Error` spanning a specific token.
    pub fn to_syn_error_spanning<T: quote::ToTokens>(&self, tokens: &T) -> syn::Error {
        syn::Error::new_spanned(tokens, self.to_string())
    }
}
