use proc_macro2::Span;
use thiserror::Error;

/// Centralized error type for Quoin macro parsing.
///
/// We use `thiserror` for clean definitions, and `manyhow` automatically
/// converts these into beautiful `compile_error!` messages with exact spans.
#[derive(Error, Debug)]
pub enum QuoinError {
    #[error("missing required attribute `{attr}`")]
    MissingAttribute {
        attr: &'static str,
        span: Span,
    },

    #[error("state field `{name}` requires a default value")]
    StateRequiresDefault {
        name: String,
        #[help("try adding `= <default_value>` after the type")]
        span: Span,
    },

    #[error("unexpected identifier `{found}`")]
    UnexpectedIdent {
        found: String,
        #[help("expected 'props', 'state', 'render', or a function definition")]
        span: Span,
    },

    #[error("expected `:` after argument name `{key}`")]
    MissingColon {
        key: String,
        span: Span,
    },

    #[error("missing render block")]
    MissingRenderBlock {
        #[help("add a `render { ... }` block to your component")]
        span: Span,
    },

    #[error("unknown Tailwind class: `{class}`")]
    UnknownTailwindClass {
        class: String,
        #[help("check the Quoin documentation for supported utility classes")]
        span: Span,
    },
}

impl QuoinError {
    /// Helper to easily convert to `syn::Error`, which `manyhow` then turns into `compile_error!`
    pub fn to_syn_error(&self) -> syn::Error {
        syn::Error::new(self.span(), self.to_string())
    }
}
