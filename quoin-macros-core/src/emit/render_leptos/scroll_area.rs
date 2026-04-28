use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node};

pub(crate) fn emit_scroll_area(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        let class_expr = find_arg_expr(el, "class");
        let mut children: Vec<TokenStream> = Vec::new();
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }

        let class_prop = if let Some(cls) = class_expr {
            quote! { class={ #cls.into() } }
        } else {
            quote! {}
        };

        let sa_alias = quote::format_ident!("ScrollArea_{}", next_extract_id());
        bindings.push(quote! {
            let #sa_alias = leptos_shadcn_ui::ScrollArea;
        });

        quote! { <#sa_alias #class_prop> #(#children)* </#sa_alias> }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let direction = find_arg_string(el, "direction").unwrap_or_else(|| "vertical".to_string());

        let overflow_class = match direction.as_str() {
            "horizontal" => "overflow-x-auto",
            "both" => "overflow-auto",
            _ => "overflow-y-auto",
        };

        let mut attrs: Vec<TokenStream> = Vec::new();
        for arg in &el.args {
            let key_str = arg.key.to_string();
            let value = &arg.value;
            match key_str.as_str() {
                "class" => attrs.push(quote! { class=format!("{} {}", #value, #overflow_class) }),
                "direction" => {}
                _ => {}
            }
        }
        if attrs.is_empty() {
            attrs.push(quote! { class=#overflow_class });
        }

        let mut children: Vec<TokenStream> = Vec::new();
        for child in &el.children {
            children.push(emit_node(child, bindings, inside_for));
        }
        quote! { <div #(#attrs)*> #(#children)* </div> }
    }
}
