use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Result, Token, braced, bracketed, parenthesized};

#[derive(Debug)]
pub enum RenderNode {
    Element(Element),
    Text(LitStr),
    Expr(Expr),
    If(IfNode),
    For(ForNode),
    Root(Vec<RenderNode>), // Added Root variant
}

#[derive(Debug)]
pub struct ArgPair {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for ArgPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
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
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;

        let args_content;
        parenthesized!(args_content in input);

        let mut args = Vec::new();
        let mut children_expr = None;

        while !args_content.is_empty() {
            let key: Ident = args_content.parse()?;
            args_content.parse::<Token![:]>()?;

            if key == "children" {
                children_expr = Some(args_content.parse()?);
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

        Ok(Element {
            name,
            args,
            children,
            children_expr,
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
        input.parse::<Ident>()?; // consume 'if'

        let condition_content;
        bracketed!(condition_content in input);
        let condition: Expr = condition_content.parse()?;

        let then_content;
        braced!(then_content in input);
        let then_branch = parse_nodes(&then_content)?;

        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;

            let fork = input.fork();
            if let Ok(next_ident) = fork.parse::<Ident>() {
                if next_ident.to_string() == "if" {
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
        input.parse::<Ident>()?; // consume 'for'

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
    "div",
    "h1",
    "h2",
    "h3",
    "p",
    "text",
    "span",
    "button",
    "input",
    "tabs",
    "tab",
    "data_table",
    "column",
];

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident: Ident = input.fork().parse()?;
            let ident_str = ident.to_string();

            // Control Flow
            if ident_str == "if" && input.peek(syn::token::Bracket) {
                return Ok(RenderNode::If(input.parse()?));
            }
            if ident_str == "for" && input.peek(syn::token::Bracket) {
                return Ok(RenderNode::For(input.parse()?));
            }

            // Known HTML Elements
            if KNOWN_ELEMENTS.contains(&ident_str.as_str()) && input.peek(syn::token::Paren) {
                return Ok(RenderNode::Element(input.parse()?));
            }

            // Macro invocations like view! { ... }
            if input.peek(Token![!]) {
                return Ok(RenderNode::Expr(input.parse()?));
            }

            // Fallback expression
            Ok(RenderNode::Expr(input.parse()?))
        } else if lookahead.peek(LitStr) {
            Ok(RenderNode::Text(input.parse()?))
        } else {
            Ok(RenderNode::Expr(input.parse()?))
        }
    }
}

fn parse_nodes(input: ParseStream) -> Result<Vec<RenderNode>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(input.parse::<RenderNode>()?);
    }
    Ok(nodes)
}
