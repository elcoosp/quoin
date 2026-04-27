use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{bindings::next_extract_id, emit_node};

pub(crate) fn emit_badge(el: &Element, bindings: &mut Vec<TokenStream>, inside_for: bool) -> TokenStream {
    let color_expr = find_arg_expr(el, "color");
    let mut children: Vec<TokenStream> = Vec::new();
    for child in &el.children {
        children.push(emit_node(child, bindings, inside_for));
    }

    #[cfg(feature = "leptos-shadcn")]
    {
        let badge_alias = quote::format_ident!("Badge_{}", next_extract_id());
        bindings.push(quote! {
            let #badge_alias = leptos_shadcn_ui::Badge;
        });

        let class_prop = if let Some(color) = color_expr {
            let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
            match bg_class {
                Some(cls) => {
                    quote! { class={format!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white {}", #cls)} }
                }
                None => {
                    quote! { class="inline-flex items-center px-1.5 rounded text-xs font-medium text-white" }
                }
            }
        } else {
            quote! { class="inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white" }
        };

        if children.is_empty() {
            quote! { <#badge_alias #class_prop /> }
        } else {
            quote! { <#badge_alias #class_prop> #(#children)* </#badge_alias> }
        }
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        match color_expr {
            Some(color) => {
                let bg_class = crate::transpile::theme_tokens::try_resolve_bg_class(color);
                match bg_class {
                    Some(cls) => quote! {
                        <span class={concat!("inline-flex items-center px-1.5 rounded text-xs font-medium text-white ", #cls)}>
                            #(#children)*
                        </span>
                    },
                    None => quote! {
                        <span
                            class="inline-flex items-center px-1.5 rounded text-xs font-medium text-white"
                            style=format!("background-color: {}", #color)
                        >
                            #(#children)*
                        </span>
                    },
                }
            }
            None => quote! {
                <span class="inline-flex items-center px-1.5 rounded text-xs font-medium bg-gray-600 text-white">
                    #(#children)*
                </span>
            },
        }
    }
}
