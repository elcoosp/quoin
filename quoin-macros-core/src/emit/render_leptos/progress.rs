use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::next_extract_id;

pub(crate) fn emit_progress(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let max_expr = find_arg_expr(el, "max");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(feature = "leptos-shadcn")]
    {
        let tag = {
            let alias = quote::format_ident!("Progress_{}", next_extract_id());
            let comp_ident = quote::format_ident!("Progress");
            bindings.push(quote! { let #alias = leptos_shadcn_ui::#comp_ident; });
            alias
        };
        let value_prop = match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) },
                    None => quote! { (#val as f64) / 100.0 },
                };
                quote! { value={leptos::prelude::Signal::derive(move || #max)} }
            }
            None => quote! {},
        };
        let class_prop = if user_class.is_empty() {
            quote! {}
        } else {
            quote! { class={ #user_class.into() } }
        };
        quote! { <#tag #value_prop #class_prop /> }
    }

    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let outer_cls = if user_class.is_empty() {
            "relative h-4 w-full overflow-hidden rounded-full bg-secondary".to_string()
        } else {
            format!(
                "relative h-4 w-full overflow-hidden rounded-full bg-secondary {}",
                user_class
            )
        };
        let bar_cls = "h-full rounded-full bg-primary transition-all duration-300";

        match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) * 100.0 },
                    None => quote! { (#val as f64) },
                };
                let val_id = next_extract_id();
                let val_name = quote::format_ident!("__quoin_prog_val_{}", val_id);
                bindings.push(quote! { let #val_name = #max; });
                quote! {
                    <div class=#outer_cls>
                        <div class=#bar_cls style={leptos::prelude::Signal::derive(move || format!("width: {}%", #val_name))} />
                    </div>
                }
            }
            None => {
                let indeterminate_cls =
                    "h-full w-1/3 rounded-full bg-primary animate-indeterminate";
                quote! {
                    <div class=#outer_cls>
                        <div class=#indeterminate_cls />
                    </div>
                }
            }
        }
    }
}
