use proc_macro2::Ident;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Block, Expr, ItemFn, Result, Token, Type, Visibility, braced};

#[derive(Debug, Clone)]
/// A `globals { name: Type observe? }` entry.
pub struct GlobalField {
    /// Global name as written.
    pub name: Ident,
    /// Declared type of the global.
    pub ty: Type,
    /// Whether the `observe` keyword requests reactivity on changes.
    pub observe: bool,
}

/// The full AST of a `component!` invocation.
pub struct ComponentAst {
    /// Component visibility (`pub` or empty).
    pub vis: Visibility,
    /// Component name.
    pub name: Ident,
    /// `props { .. }` entries.
    pub props: Vec<PropField>,
    /// `state { .. }` entries.
    pub state: Vec<StateField>,
    /// `globals { .. }` entries.
    pub globals: Vec<GlobalField>,
    /// Free `fn` items defined inline in the component.
    pub actions: Vec<ItemFn>,
    /// `on_mount { .. }` lifecycle block.
    pub on_mount: Option<Block>,
    /// `on_unmount { .. }` lifecycle block.
    pub on_unmount: Option<Block>,
    /// The mandatory `render { .. }` block.
    pub render: Block,
}

/// A `props { name: Type [= default] }` entry.
pub struct PropField {
    /// Prop name as written.
    pub name: Ident,
    /// Declared Rust type.
    pub ty: Type,
    /// Default expression, when the caller may omit the prop.
    pub default: Option<Expr>,
}

/// A `state { name: Type = default }` entry (defaults are mandatory).
pub struct StateField {
    /// State name as written.
    pub name: Ident,
    /// Declared Rust type.
    pub ty: Type,
    /// Initial value expression.
    pub default: Expr,
}

/// One `name: Type [= default]` entry of a `props`/`state`/`globals` block.
struct TypedField {
    name: Ident,
    ty: Type,
    default: Option<Expr>,
    /// `globals` entries accept a trailing `observe` keyword instead of a
    /// default value.
    observe: bool,
}

/// Parses one `name: Type [= default] [,]` entry; `require_default` enforces
/// the `state` rule that every field must have an initial value.
fn parse_typed_field(input: ParseStream, require_default: bool) -> Result<TypedField> {
    let name: Ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let ty: Type = input.parse()?;
    let default = if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        Some(input.parse()?)
    } else if require_default {
        return Err(syn::Error::new(
            name.span(),
            "State requires default value (e.g., count: u32 = 0)",
        ));
    } else {
        None
    };
    // `globals` fields may be followed by the `observe` keyword.
    let observe = if !require_default && default.is_none() && input.peek(syn::Ident) {
        let kw: syn::Ident = input.parse()?;
        kw == "observe"
    } else {
        false
    };
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }
    Ok(TypedField {
        name,
        ty,
        default,
        observe,
    })
}

fn parse_field_section(input: ParseStream, require_default: bool) -> Result<Vec<TypedField>> {
    let inner;
    braced!(inner in input);
    let mut fields = Vec::new();
    while !inner.is_empty() {
        fields.push(parse_typed_field(&inner, require_default)?);
    }
    Ok(fields)
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
            match key.to_string().as_str() {
                "props" => {
                    props = parse_field_section(&content, false)?
                        .into_iter()
                        .map(|field| PropField {
                            name: field.name,
                            ty: field.ty,
                            default: field.default,
                        })
                        .collect();
                }
                "state" => {
                    state = parse_field_section(&content, true)?
                        .into_iter()
                        .map(|field| StateField {
                            name: field.name,
                            ty: field.ty,
                            default: field
                                .default
                                .expect("state parser enforces a default value"),
                        })
                        .collect();
                }
                "globals" => {
                    globals = parse_field_section(&content, false)?
                        .into_iter()
                        .map(|field| GlobalField {
                            name: field.name,
                            ty: field.ty,
                            observe: field.observe,
                        })
                        .collect();
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

        Ok(Self {
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
