use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Token};

/// AST for the `effect!` macro.
///
/// Supports two syntaxes:
///
/// **Legacy:** `effect! { watch: [dep1, dep2], || body }`
///
/// **Structured:** `effect! { deps: [dep1, dep2], run: || body, cleanup: || cleanup }`
pub struct Effect {
    pub deps: Vec<Ident>,
    pub body: Expr,
    pub cleanup: Option<Expr>,
}

impl Parse for Effect {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: Ident = input.parse()?;

        match kw.to_string().as_str() {
            "watch" | "deps" => {
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

                let fork = input.fork();
                let next_kw: Result<Ident, _> = fork.parse();

                if let Ok(next_ident) = next_kw {
                    if next_ident == "run" {
                        input.parse::<Ident>()?;
                        input.parse::<Token![:]>()?;
                        let body: Expr = input.parse()?;

                        let mut cleanup = None;
                        if input.peek(Token![,]) {
                            input.parse::<Token![,]>()?;
                            let cleanup_kw: Ident = input.parse()?;
                            if cleanup_kw != "cleanup" {
                                return Err(syn::Error::new(
                                    cleanup_kw.span(),
                                    "expected `cleanup`",
                                ));
                            }
                            input.parse::<Token![:]>()?;
                            cleanup = Some(input.parse()?);
                        }

                        Ok(Effect {
                            deps,
                            body,
                            cleanup,
                        })
                    } else {
                        let body: Expr = input.parse()?;
                        Ok(Effect {
                            deps,
                            body,
                            cleanup: None,
                        })
                    }
                } else {
                    let body: Expr = input.parse()?;
                    Ok(Effect {
                        deps,
                        body,
                        cleanup: None,
                    })
                }
            }
            _ => Err(syn::Error::new(kw.span(), "expected `watch` or `deps`")),
        }
    }
}
