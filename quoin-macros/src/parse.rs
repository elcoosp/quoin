// quoin-macros/src/parse.rs
use proc_macro2::Ident;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Block, Expr, ItemFn, Result, Token, Type, Visibility, braced};

pub struct ComponentAst {
    pub vis: Visibility, // <-- 新增字段
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
        // 1. 先解析可见性修饰符 (pub, pub(crate) 等)
        let vis: Visibility = input.parse()?;

        // 2. 解析组件名称
        let name: Ident = input.parse()?;
        let name_span = name.span();
        let content;
        braced!(content in input);

        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut actions = Vec::new();
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
                                "State requires default value (e.g., `count: u32 = 0`)",
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
                        "Unexpected identifier. Expected 'props', 'state', 'render', or a function definition",
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
                "Missing render block. Add `render { ... }` to your component",
            ));
        }

        Ok(ComponentAst {
            vis, // <-- 返回可见性
            name,
            props,
            state,
            actions,
            render: render_block.unwrap(),
        })
    }
}
