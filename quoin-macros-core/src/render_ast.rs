//! Render AST — parsing and data structures for `quoin_render!`.
//!
//! This module defines the [`RenderNode`] enum, which is the intermediate
//! representation produced by parsing `quoin_render!` input. Each variant
//! corresponds to a syntactic construct in the macro DSL:
//!
//! - [`RenderNode::Element`] — an HTML-like element with arguments and children
//!   (e.g., `div(class: "flex") { "Hello" }`).
//! - [`RenderNode::Text`] — a string literal child (e.g., `"Hello"`).
//! - [`RenderNode::Expr`] — an arbitrary Rust expression wrapped in braces
//!   (e.g., `{my_expr}`).
//! - [`RenderNode::If`] — a conditional branch with optional else/else-if chains
//!   (e.g., `if[cond] { … } else { … }`).
//! - [`RenderNode::For`] — a loop over an iterable
//!   (e.g., `for[item in list] { … }`).
//! - [`RenderNode::Root`] — a sequence of top-level nodes (the entire macro body).
//!
//! # Parsing Flow
//!
//! 1. The `quoin_render!` proc-macro receives raw tokens and calls
//!    `syn::parse::<RenderNode>(input)`.
//! 2. [`RenderNode::parse`] inspects the next token(s) to decide which variant
//!    to parse:
//!    - `if` followed by `[` → [`IfNode`]
//!    - `for` followed by `[` → [`ForNode`]
//!    - A string literal → [`RenderNode::Text`]
//!    - An identifier followed by `(` that matches a known element name → [`Element`]
//!    - Anything else → [`RenderNode::Expr`]
//! 3. [`Element::parse`] reads outer attributes, the element name, a parenthesized
//!    argument list (`key: value, …`), and an optional brace-delimited children block.
//! 4. [`parse_nodes`] is called recursively to fill children lists.
//!
//! After parsing, the `RenderNode` tree is passed to a framework-specific emit
//! function (see [`crate::emit`]) that converts it into framework-native Rust code.

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Expr, Ident, LitStr, Result, Token, braced, bracketed, parenthesized};

#[derive(Debug)]
pub enum RenderNode {
    Element(Box<Element>),
    Text(LitStr),
    Expr(Expr),
    If(IfNode),
    For(ForNode),
    Root(Vec<RenderNode>),
}

impl RenderNode {
    pub fn attrs(&self) -> &[Attribute] {
        match self {
            RenderNode::Element(el) => &el.attrs,
            RenderNode::If(if_node) => &if_node.attrs,
            RenderNode::For(for_node) => &for_node.attrs,
            RenderNode::Text(_) | RenderNode::Expr(_) | RenderNode::Root(_) => &[],
        }
    }
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
        let value = collect_arg_value(input)?;
        Ok(ArgPair { key, value })
    }
}

#[derive(Debug)]
pub struct Element {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub args: Vec<ArgPair>,
    pub children: Vec<RenderNode>,
    pub children_expr: Option<Expr>,
    pub trigger_expr: Option<Expr>,
}

fn collect_arg_value(input: ParseStream) -> Result<Expr> {
    let mut tokens = Vec::new();
    while !input.is_empty() {
        if input.peek(Token![,]) {
            break;
        }
        let tt: proc_macro2::TokenTree = input.parse()?;
        tokens.push(tt);
    }
    let token_stream: proc_macro2::TokenStream = tokens.into_iter().collect();
    let wrapped: proc_macro2::TokenStream = quote::quote! { ( #token_stream ) };
    match syn::parse2::<Expr>(wrapped) {
        Ok(expr) => {
            let inner = match expr {
                Expr::Paren(paren) => *paren.expr,
                other => other,
            };
            Ok(inner)
        }
        Err(e) => Err(e),
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.call(Ident::parse_any)?;

        let args_content;
        parenthesized!(args_content in input);

        let mut args = Vec::new();
        let mut children_expr = None;
        let mut trigger_expr = None;

        while !args_content.is_empty() {
            let key: Ident = args_content.call(Ident::parse_any)?;
            args_content.parse::<Token![:]>()?;
            let value = collect_arg_value(&args_content)?;
            if key == "children" {
                children_expr = Some(value);
            } else if key == "trigger" {
                trigger_expr = Some(value);
            } else {
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
            if let Some(w) = warns.into_iter().next() {
                return Err(syn::Error::new_spanned(&name, w));
            }
        }

        Ok(Element {
            attrs,
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
    pub attrs: Vec<Attribute>,
    pub condition: Expr,
    pub then_branch: Vec<RenderNode>,
    pub else_branch: Option<Vec<RenderNode>>,
}

impl Parse for IfNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
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
            attrs,
            condition,
            then_branch,
            else_branch,
        })
    }
}

#[derive(Debug)]
pub struct ForNode {
    pub attrs: Vec<Attribute>,
    pub pat: Ident,
    pub iterable: Expr,
    pub body: Vec<RenderNode>,
}

impl Parse for ForNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
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
            attrs,
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
    "tabs",
    "tab",
    "data_table",
    "column",
    "virtual_list",
    "dropdown_menu",
    "rich_text",
    "clipboard_button",
    "item",
    "badge",
    "badge",
    "styled_text",
    "icon",
    "scroll_area",
    "tab_bar",
    "separator",
    "skeleton",
    "skeleton_text",
    "skeleton_avatar",
    "progress",
];

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![#]) {
            let fork = input.fork();
            if fork.call(Attribute::parse_outer).is_ok() {
                if fork.peek(Token![if]) {
                    let fork2 = fork.fork();
                    fork2.parse::<Token![if]>()?;
                    if fork2.peek(syn::token::Bracket) {
                        return Ok(RenderNode::If(input.parse()?));
                    }
                }
                if fork.peek(Token![for]) {
                    let fork2 = fork.fork();
                    fork2.parse::<Token![for]>()?;
                    if fork2.peek(syn::token::Bracket) {
                        return Ok(RenderNode::For(input.parse()?));
                    }
                }
                if fork.peek(Ident) || fork.peek(Ident::peek_any) {
                    return Ok(RenderNode::Element(input.parse()?));
                }
            }
        }

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

                let result = input.parse::<Expr>();
                return Ok(RenderNode::Expr(result?));
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
