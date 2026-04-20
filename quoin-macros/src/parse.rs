use proc_macro2::Ident;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Block, Expr, ItemFn, Result, Token, Type, braced};

pub struct ComponentAst {
    pub name: Ident,
    pub props: Vec<PropField>,
    pub state: Vec<StateField>,
    pub actions: Vec<ItemFn>,
    pub render: Block,
}

pub struct PropField {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

pub struct StateField {
    pub name: Ident,
    pub ty: Type,
    pub default: Expr,
}

impl Parse for ComponentAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);

        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut actions = Vec::new();
        let mut render_block = None;

        while !content.is_empty() {
            // Try to parse a function first
            let fork = content.fork();
            if fork.parse::<Token![fn]>().is_ok() {
                let func: ItemFn = content.parse()?;
                actions.push(func);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
                continue;
            }

            let key: Ident = content.call(Ident::parse_any)?;
            let key_str = key.to_string();

            match key_str.as_str() {
                "props" => {
                    let inner;
                    braced!(inner in content);
                    while !inner.is_empty() {
                        let fname: Ident = inner.parse()?;
                        inner.parse::<Token![:]>()?;
                        let fty: Type = inner.parse()?;
                        let default = if inner.peek(Token![=]) {
                            inner.parse::<Token![=]>()?;
                            Some(inner.parse()?)
                        } else {
                            None
                        };
                        if inner.peek(Token![,]) {
                            inner.parse::<Token![,]>()?;
                        }
                        props.push(PropField {
                            name: fname,
                            ty: fty,
                            default,
                        });
                    }
                }
                "state" => {
                    let inner;
                    braced!(inner in content);
                    while !inner.is_empty() {
                        let fname: Ident = inner.parse()?;
                        inner.parse::<Token![:]>()?;
                        let fty: Type = inner.parse()?;
                        let default = if inner.peek(Token![=]) {
                            inner.parse::<Token![=]>()?;
                            inner.parse()?
                        } else {
                            return Err(syn::Error::new(
                                fname.span(),
                                "State requires default value",
                            ));
                        };
                        if inner.peek(Token![,]) {
                            inner.parse::<Token![,]>()?;
                        }
                        state.push(StateField {
                            name: fname,
                            ty: fty,
                            default,
                        });
                    }
                }
                "render" => {
                    render_block = Some(content.parse::<Block>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unexpected identifier: {}", key_str),
                    ));
                }
            }
            // Consume optional trailing comma
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        let render =
            render_block.ok_or_else(|| syn::Error::new(name.span(), "Missing render block"))?;

        Ok(ComponentAst {
            name,
            props,
            state,
            actions,
            render,
        })
    }
}
