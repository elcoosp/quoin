use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Result, Token, braced, bracketed, parenthesized};

#[derive(Debug)]
pub enum RenderNode {
    Element(Element),
    Text(LitStr),
    Expr(Expr),
    If(IfNode),
    For(ForNode),
    Root(Vec<RenderNode>),
}

#[derive(Debug)]
pub struct ArgPair {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for ArgPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.call(Ident::parse_any)?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;
        Ok(ArgPair { key, value })
    }
}

#[derive(Debug)]
pub struct Element {
    pub name: Ident,
    pub args: Vec<ArgPair>,
    pub children: Vec<RenderNode>,
    pub children_expr: Option<Expr>,
    pub trigger_expr: Option<Expr>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.call(Ident::parse_any)?;

        let args_content;
        parenthesized!(args_content in input);

        let mut args = Vec::new();
        let mut children_expr = None;
        let mut trigger_expr = None;

        while !args_content.is_empty() {
            let key: Ident = args_content.call(Ident::parse_any)?;
            args_content.parse::<Token![:]>()?;

            if key == "children" {
                children_expr = Some(args_content.parse()?);
            } else if key == "trigger" {
                trigger_expr = Some(args_content.parse()?);
            } else {
                let value: Expr = args_content.parse()?;
                args.push(ArgPair { key, value });
            }

            if !args_content.is_empty() {
                args_content.parse::<Token![,]>()?;
            }
        }

        let children = if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            parse_nodes(&content)?
        } else {
            Vec::new()
        };

        {
            let arg_keys: Vec<&Ident> = args.iter().map(|a| &a.key).collect();
            let warns = crate::render_ast_diag::check_element_args(&name.to_string(), &arg_keys);
            for w in warns {
                return Err(syn::Error::new_spanned(&name, w));
            }
        }

        Ok(Element {
            name,
            args,
            children,
            children_expr,
            trigger_expr,
        })
    }
}

#[derive(Debug)]
pub struct IfNode {
    pub condition: Expr,
    pub then_branch: Vec<RenderNode>,
    pub else_branch: Option<Vec<RenderNode>>,
}

impl Parse for IfNode {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![if]>()?;

        let condition_content;
        bracketed!(condition_content in input);
        let condition: Expr = condition_content.parse()?;

        let then_content;
        braced!(then_content in input);
        let then_branch = parse_nodes(&then_content)?;

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;

            if input.peek(Token![if]) {
                let fork = input.fork();
                fork.parse::<Token![if]>()?;
                if fork.peek(syn::token::Bracket) {
                    let nested_if = input.parse::<IfNode>()?;
                    Some(vec![RenderNode::If(nested_if)])
                } else {
                    let else_content;
                    braced!(else_content in input);
                    Some(parse_nodes(&else_content)?)
                }
            } else {
                let else_content;
                braced!(else_content in input);
                Some(parse_nodes(&else_content)?)
            }
        } else {
            None
        };

        Ok(IfNode {
            condition,
            then_branch,
            else_branch,
        })
    }
}

#[derive(Debug)]
pub struct ForNode {
    pub pat: Ident,
    pub iterable: Expr,
    pub body: Vec<RenderNode>,
}

impl Parse for ForNode {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![for]>()?;

        let for_content;
        bracketed!(for_content in input);
        let pat: Ident = for_content.parse()?;
        for_content.parse::<Token![in]>()?;
        let iterable: Expr = for_content.parse()?;

        let body_content;
        braced!(body_content in input);
        let body = parse_nodes(&body_content)?;

        Ok(ForNode {
            pat,
            iterable,
            body,
        })
    }
}

const KNOWN_ELEMENTS: &[&str] = &[
    // Standard HTML
    "div",
    "h1",
    "h2",
    "h3",
    "p",
    "text",
    "span",
    "button",
    "input",
    "label",
    "img",
    "a",
    "ul",
    "ol",
    "li",
    "hr",
    "br",
    "textarea",
    "select",
    "form",
    // UCP components
    "tabs",
    "tab",
    "data_table",
    "column",
    "virtual_list",
    "dropdown_menu",
    "rich_text",
    "clipboard_button",
    "item",
    // Aliases
    "tab_bar",
];

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![if]) {
            let fork = input.fork();
            fork.parse::<Token![if]>()?;
            if fork.peek(syn::token::Bracket) {
                return Ok(RenderNode::If(input.parse()?));
            }
        }

        if input.peek(Token![for]) {
            let fork = input.fork();
            fork.parse::<Token![for]>()?;
            if fork.peek(syn::token::Bracket) {
                return Ok(RenderNode::For(input.parse()?));
            }
        }

        if input.peek(LitStr) {
            return Ok(RenderNode::Text(input.parse()?));
        }

        if input.peek(Ident) || input.peek(Ident::peek_any) {
            let fork = input.fork();
            if let Ok(ident) = fork.call(Ident::parse_any) {
                let ident_str = ident.to_string();

                if fork.peek(syn::token::Paren) {
                    if !KNOWN_ELEMENTS.contains(&ident_str.as_str()) {
                        let suggestion = crate::render_ast_diag::suggest_element(&ident_str);
                        let msg = if let Some(sug) = suggestion {
                            format!(
                                "unknown element `{}`. Did you mean `{}`? Known elements: {}",
                                ident_str,
                                sug,
                                KNOWN_ELEMENTS.join(", ")
                            )
                        } else {
                            format!(
                                "unknown element `{}`. Known elements: {}. If this is a function call, wrap it in braces: `{{ expr }}`",
                                ident_str,
                                KNOWN_ELEMENTS.join(", ")
                            )
                        };
                        return Err(syn::Error::new_spanned(ident, msg));
                    }
                    return Ok(RenderNode::Element(input.parse()?));
                }

                return Ok(RenderNode::Expr(input.parse()?));
            }
        }

        Ok(RenderNode::Expr(input.parse()?))
    }
}

fn parse_nodes(input: ParseStream) -> Result<Vec<RenderNode>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(input.parse::<RenderNode>()?);
    }
    Ok(nodes)
}
