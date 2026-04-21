//! Parser for the `run_app!` macro.
//!
//! Syntax:
//! ```
//! run_app!(ComponentName)
//! run_app!(ComponentName, window_opts: expr)
//! ```

use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

pub struct RunAppInput {
    pub component: Ident,
    pub window_opts: Option<syn::Expr>,
}

impl Parse for RunAppInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let component: Ident = input.parse()?;

        let window_opts = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            // Expect "window_opts:"
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

        Ok(RunAppInput {
            component,
            window_opts,
        })
    }
}
