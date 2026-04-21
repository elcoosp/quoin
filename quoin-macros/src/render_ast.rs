use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Result, Token, braced, parenthesized};

#[derive(Debug)]
pub enum RenderNode {
    Element(Element),
    Text(LitStr),
    Expr(Expr),
    If(IfNode),
    For(ForNode),
}

#[derive(Debug)]
pub struct Element {
    pub name: Ident,
    pub args: Vec<(Ident, Expr)>,
    pub children: Vec<RenderNode>,
    pub children_expr: Option<Expr>,
}

#[derive(Debug)]
pub struct IfNode {
    pub condition: Expr,
    pub then_branch: Vec<RenderNode>,
    pub else_branch: Option<Vec<RenderNode>>,
}

#[derive(Debug)]
pub struct ForNode {
    pub pat: Ident,
    pub iterable: Expr,
    pub body: Vec<RenderNode>,
}

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident: Ident = input.fork().parse()?;
            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "@if" => {
                    input.parse::<Ident>()?; // consume @if
                    let condition: Expr = input.parse()?;
                    let then_content;
                    braced!(then_content in input);
                    let then_branch = parse_nodes(&then_content)?;

                    let else_branch = if input.peek(Token![else]) {
                        input.parse::<Token![else]>()?;

                        // Check for `@if` immediately after `else` to support `@else if`
                        let fork = input.fork();
                        if fork.parse::<Ident>().map(|i| i.to_string()).as_deref() == Ok("@if") {
                            let nested_if = input.parse::<RenderNode>()?;
                            Some(vec![nested_if])
                        } else {
                            let else_content;
                            braced!(else_content in input);
                            Some(parse_nodes(&else_content)?)
                        }
                    } else {
                        None
                    };

                    Ok(RenderNode::If(IfNode {
                        condition,
                        then_branch,
                        else_branch,
                    }))
                }
                "@for" => {
                    input.parse::<Ident>()?; // consume @for
                    let pat: Ident = input.parse()?;
                    input.parse::<Token![in]>()?;
                    let iterable: Expr = input.parse()?;
                    let body_content;
                    braced!(body_content in input);
                    let body = parse_nodes(&body_content)?;
                    Ok(RenderNode::For(ForNode {
                        pat,
                        iterable,
                        body,
                    }))
                }
                "if" | "for_each" => Ok(RenderNode::Expr(input.parse()?)),
                _ => {
                    let fork = input.fork();
                    fork.parse::<Ident>()?;
                    if fork.peek(Token![!]) {
                        Ok(RenderNode::Expr(input.parse()?))
                    } else if fork.peek(syn::token::Paren) {
                        parse_element(input)
                    } else {
                        Ok(RenderNode::Expr(input.parse()?))
                    }
                }
            }
        } else if lookahead.peek(LitStr) {
            Ok(RenderNode::Text(input.parse()?))
        } else {
            Ok(RenderNode::Expr(input.parse()?))
        }
    }
}

fn parse_element(input: ParseStream) -> Result<RenderNode> {
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
            args.push((key, value));
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
    Ok(RenderNode::Element(Element {
        name,
        args,
        children,
        children_expr,
    }))
}

fn parse_nodes(input: ParseStream) -> Result<Vec<RenderNode>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(input.parse::<RenderNode>()?);
    }
    Ok(nodes)
}
