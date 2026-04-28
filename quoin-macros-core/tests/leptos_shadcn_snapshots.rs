#![cfg(all(feature = "leptos", feature = "leptos-shadcn"))]

use proc_macro2::TokenStream;
use quoin_macros_core::render_ast::{Element, RenderNode, ArgPair};

fn el(name: &str, args: Vec<(&str, syn::Expr)>, children: Vec<RenderNode>) -> RenderNode {
    RenderNode::Element(Box::new(Element {
        attrs: vec![],
        name: syn::Ident::new(name, proc_macro2::Span::call_site()),
        args: args.into_iter().map(|(k, v)| ArgPair {
            key: syn::Ident::new(k, proc_macro2::Span::call_site()),
            value: v,
        }).collect(),
        children,
        children_expr: None,
        trigger_expr: None,
    }))
}

fn emit(node: &RenderNode) -> TokenStream {
    quoin_macros_core::emit::render_leptos::emit_render(node)
}

fn str_expr(s: &str) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: syn::Lit::Str(syn::LitStr::new(s, proc_macro2::Span::call_site())),
    })
}
fn ident_expr(name: &str) -> syn::Expr {
    syn::Expr::Path(syn::ExprPath {
        attrs: vec![],
        qself: None,
        path: syn::Ident::new(name, proc_macro2::Span::call_site()).into(),
    })
}
fn bool_expr(val: bool) -> syn::Expr {
    syn::Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: syn::Lit::Bool(syn::LitBool::new(val, proc_macro2::Span::call_site())),
    })
}
fn txt(s: &str) -> RenderNode {
    RenderNode::Text(syn::LitStr::new(s, proc_macro2::Span::call_site()))
}

fn contains(ts: &TokenStream, needle: &str) -> bool {
    ts.to_string().contains(needle)
}

#[test]
fn test_label() {
    let ts = emit(&el("label", vec![], vec![]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Label"));
}

#[test]
fn test_textarea() {
    let ts = emit(&el("textarea", vec![("placeholder", str_expr("Enter text"))], vec![]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Textarea"));
}

#[test]
fn test_toggle() {
    let ts = emit(&el("toggle", vec![("pressed", ident_expr("is_on"))], vec![]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Toggle"));
}

#[test]
fn test_card_with_header() {
    let card_title = el("card_title", vec![], vec![txt("Title")]);
    let card_header = el("card_header", vec![], vec![card_title]);
    let ts = emit(&el("card", vec![("variant", str_expr("warning"))], vec![card_header]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Card"));
    assert!(contains(&ts, "leptos_shadcn_ui :: CardHeader"));
    assert!(contains(&ts, "leptos_shadcn_ui :: CardTitle"));
}

#[test]
fn test_dialog() {
    let content = el("dialog_content", vec![], vec![txt("Content")]);
    let ts = emit(&el("dialog", vec![("open", ident_expr("my_signal"))], vec![content]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Dialog"));
    assert!(contains(&ts, "leptos_shadcn_ui :: DialogContent"));
}

#[test]
fn test_accordion() {
    let trigger = el("accordion_trigger", vec![], vec![txt("Trigger")]);
    let content = el("accordion_content", vec![("force_mount", bool_expr(true))], vec![txt("Content")]);
    let item = el("accordion_item", vec![("value", str_expr("item1"))], vec![trigger, content]);
    let ts = emit(&el("accordion", vec![("value", ident_expr("my_value_signal"))], vec![item]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Accordion"));
    assert!(contains(&ts, "leptos_shadcn_ui :: AccordionItem"));
    assert!(contains(&ts, "leptos_shadcn_ui :: AccordionTrigger"));
    assert!(contains(&ts, "leptos_shadcn_ui :: AccordionContent"));
}

#[test]
fn test_context_menu() {
    let trigger = el("context_menu_trigger", vec![], vec![txt("Right Click")]);
    let item = el("context_menu_item", vec![("class", str_expr("item"))], vec![txt("Action")]);
    let content = el("context_menu_content", vec![], vec![item]);
    let ts = emit(&el("context_menu", vec![], vec![trigger, content]));
    assert!(contains(&ts, "leptos_shadcn_ui :: ContextMenu"));
    assert!(contains(&ts, "leptos_shadcn_ui :: ContextMenuTrigger"));
    assert!(contains(&ts, "leptos_shadcn_ui :: ContextMenuItem"));
}

#[test]
fn test_command() {
    let input = el("command_input", vec![("placeholder", str_expr("Search..."))], vec![]);
    let item = el("command_item", vec![("value", str_expr("opt1"))], vec![txt("Option 1")]);
    let list = el("command_list", vec![], vec![item]);
    let ts = emit(&el("command", vec![], vec![input, list]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Command"));
    assert!(contains(&ts, "leptos_shadcn_ui :: CommandInput"));
    assert!(contains(&ts, "leptos_shadcn_ui :: CommandItem"));
}

#[test]
fn test_form() {
    let label = el("form_label", vec![("for_field", str_expr("email"))], vec![txt("Email")]);
    let control = el("form_control", vec![], vec![]);
    let message = el("form_message", vec![("message", str_expr("Invalid email"))], vec![]);
    let field = el("form_field", vec![("name", str_expr("email"))], vec![label, control, message]);
    let ts = emit(&el("form", vec![("on_submit", ident_expr("handle_submit"))], vec![field]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Form"));
    assert!(contains(&ts, "leptos_shadcn_ui :: FormField"));
    assert!(contains(&ts, "leptos_shadcn_ui :: FormLabel"));
    assert!(contains(&ts, "leptos_shadcn_ui :: FormControl"));
    assert!(contains(&ts, "leptos_shadcn_ui :: FormMessage"));
}

#[test]
fn test_select() {
    let trigger = el("select_trigger", vec![], vec![txt("Choose")]);
    let item = el("select_item", vec![("value", str_expr("a"))], vec![txt("A")]);
    let content = el("select_content", vec![], vec![item]);
    let ts = emit(&el("select", vec![("value", ident_expr("selected"))], vec![trigger, content]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Select"));
    assert!(contains(&ts, "leptos_shadcn_ui :: SelectTrigger"));
    assert!(contains(&ts, "leptos_shadcn_ui :: SelectItem"));
}

#[test]
fn test_toast_provider() {
    let ts = emit(&el("toast_provider", vec![], vec![]));
    assert!(contains(&ts, "leptos_shadcn_ui :: Toaster"));
}

#[test]
fn test_error_boundary() {
    let ts = emit(&el("error_boundary", vec![], vec![txt("fallback")]));
    assert!(contains(&ts, "leptos_shadcn_ui :: ErrorBoundary"));
}

#[test]
fn test_lazy_component() {
    let ts = emit(&el("lazy_component", vec![("name", str_expr("MyComp"))], vec![]));
    assert!(contains(&ts, "leptos_shadcn_ui :: LazyComponent"));
}
