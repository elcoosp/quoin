use crate::emit::common::find_arg_expr;
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node, generic::emit_html_tag};

pub(crate) fn emit_clipboard_button(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    let copy_text = find_arg_expr(el, "copy_text");
    match copy_text {
        Some(ct) => {
            let clip_id = next_extract_id();
            let clip_name = quote::format_ident!("__quoin_clip_{}", clip_id);
            bindings.push(quote! {
                let #clip_name = {
                    let __ct: String = (#ct).to_string();
                    move |_: leptos::ev::MouseEvent| {
                        quoin::clipboard_write_text(&__ct);
                    }
                };
            });
            let mut attrs: Vec<TokenStream> = Vec::new();
            for arg in &el.args {
                let key_str = arg.key.to_string();
                let value = &arg.value;
                match key_str.as_str() {
                    "class" => attrs.push(quote! { class=#value }),
                    "id" => attrs.push(quote! { id=#value }),
                    "disabled" => attrs.push(quote! { disabled=#value }),
                    _ => {}
                }
            }
            let mut children: Vec<TokenStream> = Vec::new();
            for child in &el.children {
                children.push(emit_node(child, bindings, inside_for));
            }
            let tag_ident = proc_macro2::Ident::new("button", proc_macro2::Span::call_site());
            if children.is_empty() {
                quote! { <#tag_ident #(#attrs)* on:click=#clip_name /> }
            } else {
                quote! { <#tag_ident #(#attrs)* on:click=#clip_name> #(#children)* </#tag_ident> }
            }
        }
        None => emit_html_tag(el, "button", bindings, inside_for),
    }
}
