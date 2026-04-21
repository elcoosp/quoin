use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Token};

/// AST for the `effect!` macro.
///
/// Syntax: `effect! { watch: [dep1, dep2, ...], || body }`
pub struct Effect {
    pub deps: Vec<Ident>,
    pub body: Expr,
}

impl Parse for Effect {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: Ident = input.parse()?;
        if kw != "watch" {
            return Err(syn::Error::new(kw.span(), "expected `watch`"));
        }
        input.parse::<Token![:]>()?;
        let deps_content;
        syn::bracketed!(deps_content in input);
        let mut deps = Vec::new();
        while !deps_content.is_empty() {
            deps.push(deps_content.parse::<Ident>()?);
            if deps_content.peek(Token![,]) {
                deps_content.parse::<Token![,]>()?;
            }
        }
        input.parse::<Token![,]>()?;
        let body: Expr = input.parse()?;
        Ok(Effect { deps, body })
    }
}
