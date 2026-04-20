use syn::parse::{Parse, ParseStream};
use syn::{braced, Ident, Type, Expr, Block, ItemFn, Result};
use syn::ext::IdentExt;

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
            let lookahead = content.lookahead1();
            if lookahead.peek(Ident) {
                let key: Ident = content.call(Ident::parse_any)?;
                let key_str = key.to_string();

                match key_str.as_str() {
                    "props" => {
                        let inner;
                        braced!(inner in content);
                        while !inner.is_empty() {
                            let fname: Ident = inner.parse()?;
                            inner.parse::<syn::Token![:]>()?;
                            let fty: Type = inner.parse()?;
                            let default = if inner.peek(syn::Token![=]) {
                                inner.parse::<syn::Token![=]>()?;
                                Some(inner.parse()?)
                            } else {
                                None
                            };
                            if !inner.is_empty() {
                                inner.parse::<syn::Token![,]>()?;
                            }
                            props.push(PropField { name: fname, ty: fty, default });
                        }
                    }
                    "state" => {
                        let inner;
                        braced!(inner in content);
                        while !inner.is_empty() {
                            let fname: Ident = inner.parse()?;
                            inner.parse::<syn::Token![:]>()?;
                            let fty: Type = inner.parse()?;
                            let default = if inner.peek(syn::Token![=]) {
                                inner.parse::<syn::Token![=]>()?;
                                inner.parse()?
                            } else {
                                return Err(syn::Error::new(fname.span(), "State requires default value"));
                            };
                            if !inner.is_empty() {
                                inner.parse::<syn::Token![,]>()?;
                            }
                            state.push(StateField { name: fname, ty: fty, default });
                        }
                    }
                    "render" => {
                        let inner;
                        braced!(inner in content);
                        render_block = Some(inner.parse::<Block>()?);
                    }
                    _ => {
                        let func: ItemFn = content.parse()?;
                        actions.push(func);
                    }
                }
            } else {
                return Err(content.error("Expected identifier or closing brace"));
            }
        }

        Ok(ComponentAst {
            name,
            props,
            state,
            actions,
            render: render_block.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "Missing render block")
            })?,
        })
    }
}
