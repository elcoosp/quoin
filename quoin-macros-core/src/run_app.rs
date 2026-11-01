//! Parser for the `run_app!` macro.
//!
//! Syntax:
//!
//! ```ignore
//! run_app!(ComponentName)
//! run_app!(ComponentName, window_opts: expr)
//! ```

use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

/// The AST of a `run_app!` invocation.
pub struct RunAppInput {
    /// The component type to bootstrap.
    pub component: Ident,
    /// Optional `window_opts: <expr>` override for window construction.
    pub window_opts: Option<syn::Expr>,
}

impl Parse for RunAppInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let component: Ident = input.parse()?;

        let window_opts = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let key: Ident = input.parse()?;
            if key != "window_opts" {
                return Err(syn::Error::new(key.span(), "expected `window_opts`"));
            }
            input.parse::<Token![:]>()?;
            let expr: syn::Expr = input.parse()?;
            Some(expr)
        } else {
            None
        };

        Ok(Self {
            component,
            window_opts,
        })
    }
}
