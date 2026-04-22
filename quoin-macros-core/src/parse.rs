use proc_macro2::Ident;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Block, Expr, ItemFn, Result, Token, Type, Visibility, braced};

#[derive(Debug, Clone)]
pub struct GlobalField {
    pub name: Ident,
    pub ty: Type,
    pub observe: bool,
}

pub struct ComponentAst {
    pub vis: Visibility,
    pub name: Ident,
    pub props: Vec<PropField>,
    pub state: Vec<StateField>,
    pub globals: Vec<GlobalField>,
    pub actions: Vec<ItemFn>,
    pub on_mount: Option<Block>,
    pub on_unmount: Option<Block>,
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
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        let name_span = name.span();
        let content;
        braced!(content in input);

        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut globals = Vec::new();
        let mut actions = Vec::new();
        let mut on_mount: Option<Block> = None;
        let mut on_unmount: Option<Block> = None;
        let mut render_block: Option<Block> = None;

        while !content.is_empty() {
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
                                "State requires default value (e.g., count: u32 = 0)",
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
                "globals" => {
                    let inner;
                    braced!(inner in content);
                    while !inner.is_empty() {
                        let fname: Ident = inner.parse()?;
                        inner.parse::<Token![:]>()?;
                        let fty: Type = inner.parse()?;
                        let observe = if inner.peek(syn::Ident) {
                            let kw: syn::Ident = inner.parse()?;
                            kw.to_string() == "observe"
                        } else {
                            false
                        };
                        if inner.peek(Token![,]) {
                            inner.parse::<Token![,]>()?;
                        }
                        globals.push(GlobalField {
                            name: fname,
                            ty: fty,
                            observe,
                        });
                    }
                }
                "on_mount" => {
                    on_mount = Some(content.parse::<Block>()?);
                }
                "on_unmount" => {
                    on_unmount = Some(content.parse::<Block>()?);
                }
                "render" => {
                    render_block = Some(content.parse::<Block>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "Unexpected identifier. Expected 'props', 'state', 'globals', 'on_mount', 'on_unmount', 'render', or a function definition",
                    ));
                }
            }
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        if render_block.is_none() {
            return Err(syn::Error::new(
                name_span,
                "Missing render block. Add render { ... } to your component",
            ));
        }

        Ok(ComponentAst {
            vis,
            name,
            props,
            state,
            globals,
            actions,
            on_mount,
            on_unmount,
            render: render_block.unwrap(),
        })
    }
}
