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
use syn::{Attribute, Expr, Ident, LitStr, Result, Stmt, Token, braced, bracketed, parenthesized};

/// A node of the declarative-UI tree parsed from `quoin_render!` input.
#[derive(Clone, Debug)]
pub enum RenderNode {
    /// An HTML-like element with arguments and children.
    Element(Box<Element>),
    /// A string literal child.
    Text(LitStr),
    /// A braced Rust expression.
    Expr(Expr),
    /// A conditional branch (`if[cond] { .. } else { .. }`).
    If(IfNode),
    /// A loop (`for[item in list] { .. }`). Boxed because `ForNode` is
    /// by far the largest variant (iterable + key + body); keeping the enum
    /// small matters for the parser's hot path.
    For(Box<ForNode>),
    /// A sequence of nodes (top-level macro body or block children).
    Root(Vec<Self>),
}

impl RenderNode {
    /// The outer attributes attached to this node (empty for text/expression
    /// nodes, which cannot carry attributes).
    #[must_use]
    pub fn attrs(&self) -> &[Attribute] {
        match self {
            Self::Element(el) => &el.attrs,
            Self::If(if_node) => &if_node.attrs,
            Self::For(for_node) => &for_node.attrs,
            Self::Text(_) | Self::Expr(_) | Self::Root(_) => &[],
        }
    }
}

#[derive(Clone, Debug)]
/// One `key: value` argument of an element.
pub struct ArgPair {
    /// Argument name.
    pub key: Ident,
    /// Argument value expression (collected greedily until `,` or `)`).
    pub value: Expr,
}

impl Parse for ArgPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.call(Ident::parse_any)?;
        input.parse::<Token![:]>()?;
        let value = collect_arg_value(input)?;
        Ok(Self { key, value })
    }
}

/// An HTML-like element: `name(key: value, ..) { children }`.
#[derive(Clone, Debug)]
pub struct Element {
    /// Outer attributes (`#[cfg(..)]` and friends).
    pub attrs: Vec<Attribute>,
    /// Element name (e.g. `div`, `button`, `tabs`).
    pub name: Ident,
    /// Parenthesized `key: value` arguments.
    pub args: Vec<ArgPair>,
    /// Brace-delimited children, parsed recursively.
    pub children: Vec<RenderNode>,
    /// When the children block is a single braced Rust expression instead of
    /// DSL nodes, it lands here (e.g. dynamic children).
    pub children_expr: Option<Expr>,
    /// Optional `trigger(...)` expression (e.g. dropdown trigger).
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

        Ok(Self {
            attrs,
            name,
            args,
            children,
            children_expr,
            trigger_expr,
        })
    }
}

/// A conditional branch parsed from `if[condition] { .. } else { .. }`.
#[derive(Clone, Debug)]
pub struct IfNode {
    /// Outer attributes (`#[cfg(..)]` and friends) attached to the `if`.
    pub attrs: Vec<Attribute>,
    /// The bracketed condition expression.
    pub condition: Expr,
    /// Nodes of the `then` block.
    pub then_branch: Vec<RenderNode>,
    /// Nodes of the `else`/`else if` block, when present.
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
                    let nested_if = input.parse::<Self>()?;
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
        Ok(Self {
            attrs,
            condition,
            then_branch,
            else_branch,
        })
    }
}

/// A loop parsed from `for[item in iterable] { .. }`.
#[derive(Clone, Debug)]
pub struct ForNode {
    /// Outer attributes attached to the `for`.
    pub attrs: Vec<Attribute>,
    /// Loop variable pattern (a plain identifier).
    pub pat: Ident,
    /// The iterable expression.
    pub iterable: Expr,
    /// Optional key expression (`for[item in list, key: item.id]`), used by
    /// diffing backends (Floem's `dyn_stack`).
    pub key: Option<Expr>,
    /// Nodes of the loop body.
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
        let key = if for_content.peek(Token![,]) {
            for_content.parse::<Token![,]>()?;
            let key_ident: Ident = for_content.call(Ident::parse_any)?;
            if key_ident != "key" {
                return Err(syn::Error::new(
                    key_ident.span(),
                    "expected `key: <expr>` after the comma (e.g. `for[item in list, key: item.id]`)",
                ));
            }
            for_content.parse::<Token![:]>()?;
            Some(for_content.parse()?)
        } else {
            None
        };
        let body_content;
        braced!(body_content in input);
        let body = parse_nodes(&body_content)?;
        Ok(Self {
            attrs,
            pat,
            iterable,
            key,
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
    "svg",
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
    "checkbox",
    "switch",
    "radio_group",
    "radio",
    "slider",
    "tooltip",
    // Rich-elements Phase 0 (canonical shadcn Tier-1 surface — see
    // `emit::shadcn_spec` and docs/quoin-shadcn-rich-elements-evaluation.md).
    // Lowered generically (containers/text) on every backend; the
    // `<framework>-shadcn` features upgrade the lowering to the real
    // component libraries.
    "card",
    "card_header",
    "card_title",
    "card_description",
    "card_content",
    "card_footer",
    "card_action",
    "alert",
    "alert_title",
    "alert_description",
    "spinner",
    "kbd",
    "avatar",
    "avatar_fallback",
    // Rich-elements Phase 2 (canonical shadcn Tier-2 — disclosure family
    // plus the canonical tabs/tooltip upgrades; see `emit::shadcn_spec`).
    "accordion",
    "accordion_item",
    "accordion_trigger",
    "accordion_content",
];

/// The accepted element names, exposed for contract tests
/// (`emit::shadcn_spec` asserts its Tier-1 table stays a subset).
pub fn known_elements() -> &'static [&'static str] {
    KNOWN_ELEMENTS
}


/// Unwraps a single-expression block: `{ expr }` -> `expr`.
///
/// Text children are authored as `{ expr }` to disambiguate them from
/// pattern bindings; splicing the block verbatim into argument position
/// trips `unused_braces`, so the parse layer keeps just the expression.
fn unblock_expr(expr: Expr) -> Expr {
    if let Expr::Block(block) = &expr {
        let stmts = &block.block.stmts;
        if stmts.len() == 1 && let Some(Stmt::Expr(inner, None)) = stmts.first() {
            return (*inner).clone();
        }
    }
    expr
}

impl Parse for RenderNode {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![#]) {
            let fork = input.fork();
            if fork.call(Attribute::parse_outer).is_ok() {
                if fork.peek(Token![if]) {
                    let fork2 = fork.fork();
                    fork2.parse::<Token![if]>()?;
                    if fork2.peek(syn::token::Bracket) {
                        return Ok(Self::If(input.parse()?));
                    }
                }
                if fork.peek(Token![for]) {
                    let fork2 = fork.fork();
                    fork2.parse::<Token![for]>()?;
                    if fork2.peek(syn::token::Bracket) {
                        return Ok(Self::For(input.parse()?));
                    }
                }
                if fork.peek(Ident) || fork.peek(Ident::peek_any) {
                    return Ok(Self::Element(input.parse()?));
                }
            }
        }

        if input.peek(Token![if]) {
            let fork = input.fork();
            fork.parse::<Token![if]>()?;
            if fork.peek(syn::token::Bracket) {
                return Ok(Self::If(input.parse()?));
            }
        }

        if input.peek(Token![for]) {
            let fork = input.fork();
            fork.parse::<Token![for]>()?;
            if fork.peek(syn::token::Bracket) {
                return Ok(Self::For(input.parse()?));
            }
        }

        if input.peek(LitStr) {
            return Ok(Self::Text(input.parse()?));
        }

        if input.peek(Ident) || input.peek(Ident::peek_any) {
            let fork = input.fork();
            if let Ok(ident) = fork.call(Ident::parse_any) {
                let ident_str = ident.to_string();

                if fork.peek(syn::token::Paren) {
                    if !KNOWN_ELEMENTS.contains(&ident_str.as_str()) {
                        let suggestion = crate::render_ast_diag::suggest_element(&ident_str);
                        let msg = suggestion.map_or_else(
                            || {
                                format!(
                                    "unknown element `{}`. Known elements: {}. If this is a function call, wrap it in braces: `{{ expr }}`",
                                    ident_str,
                                    KNOWN_ELEMENTS.join(", ")
                                )
                            },
                            |sug| {
                                format!(
                                    "unknown element `{}`. Did you mean `{}`? Known elements: {}",
                                    ident_str,
                                    sug,
                                    KNOWN_ELEMENTS.join(", ")
                                )
                            },
                        );
                        return Err(syn::Error::new_spanned(ident, msg));
                    }
                    return Ok(Self::Element(input.parse()?));
                }

                let result = input.parse::<Expr>().map(unblock_expr);
                return Ok(Self::Expr(result?));
            }
        }

        Ok(Self::Expr(input.parse::<Expr>().map(unblock_expr)?))
    }
}

fn parse_nodes(input: ParseStream) -> Result<Vec<RenderNode>> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        nodes.push(input.parse::<RenderNode>()?);
    }
    Ok(nodes)
}
