use syn::parse::{Parse, ParseStream};
use syn::{braced, parenthesized, Ident, Expr, LitStr, Result, Token};

#[derive(Debug)]
pub enum RenderNode {
    Element(Element),
    Text(LitStr),
    Expr(Expr),
    If(IfNode),
    ForEach(ForEachNode),
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
pub struct ForEachNode {
    pub items: Expr,
    pub key: Expr,
    pub item_template: Box<RenderNode>,
}

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let ident: Ident = input.fork().parse()?;
            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "if" => {
                    input.parse::<Ident>()?;
                    let condition: Expr = input.parse()?;
                    let then_content;
                    braced!(then_content in input);
                    let then_branch = parse_nodes(&then_content)?;
                    let else_branch = if input.peek(Token![else]) {
                        input.parse::<Token![else]>()?;
                        let else_content;
                        braced!(else_content in input);
                        Some(parse_nodes(&else_content)?)
                    } else { None };
                    Ok(RenderNode::If(IfNode { condition, then_branch, else_branch }))
                }
                "for_each" => {
                    input.parse::<Ident>()?;
                    let args_content;
                    parenthesized!(args_content in input);
                    let items: Expr = args_content.parse()?;
                    args_content.parse::<Token![,]>()?;
                    args_content.parse::<Ident>()?;
                    args_content.parse::<Token![:]>()?;
                    let key: Expr = args_content.parse()?;
                    let template_content;
                    braced!(template_content in input);
                    let template_node = parse_node(&template_content)?;
                    Ok(RenderNode::ForEach(ForEachNode { items, key, item_template: Box::new(template_node) }))
                }
                _ => {
                    let fork = input.fork();
                    fork.parse::<Ident>()?;
                    if fork.peek(Token![!]) {
                        // Macro call like format!, vec!
                        Ok(RenderNode::Expr(input.parse()?))
                    } else if fork.peek(syn::token::Paren) {
                        // Element with arguments
                        parse_element(input)
                    } else {
                        // Plain expression (variable, literal, etc.)
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
    } else { Vec::new() };
    Ok(RenderNode::Element(Element { name, args, children, children_expr }))
}

fn parse_nodes(input: ParseStream) -> Result<Vec<RenderNode>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(input.parse::<RenderNode>()?);
    }
    Ok(nodes)
}

fn parse_node(input: ParseStream) -> Result<RenderNode> {
    input.parse::<RenderNode>()
}
