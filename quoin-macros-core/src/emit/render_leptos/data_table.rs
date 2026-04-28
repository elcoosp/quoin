use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::next_extract_id;

pub(crate) fn emit_data_table(el: &Element, bindings: &mut Vec<TokenStream>, _inside_for: bool) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows");
    let striped = find_arg_bool(el, "striped");
    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c && e.name == "column" {
            let col_label = find_arg_expr(e, "label").unwrap_or(&empty_label);
            let width = find_arg_expr(e, "width");
            let mut th_attrs = vec![quote! { class={ "px-3 py-2 text-gray-400 font-medium".into() } }];
            if let Some(w) = width {
                th_attrs.push(quote! { style=format!("width: {}px", #w) });
            }
            header_cells.push(quote! { <th #(#th_attrs)*>{#col_label}</th> });

            let render_closure = find_arg_expr(e, "render");
            if let Some(rc) = render_closure {
                let col_id = next_extract_id();
                let render_name = quote::format_ident!("__quoin_col_{}", col_id);
                bindings.push(quote! { let #render_name = std::sync::Arc::new(#rc); });
                let mut td_attrs = vec![quote! { class={ "px-3 py-2 text-white".into() } }];
                if let Some(w) = width {
                    td_attrs.push(quote! { style=format!("width: {}px", #w) });
                }
                row_cells.push(quote! { <td #(#td_attrs)*>{ (&*#render_name)(&__row) }</td> });
            } else {
                row_cells.push(quote! { <td class={ "px-3 py-2 text-white".into() }></td> });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let striped_class = if striped { " table-striped" } else { "" };

    // Extract rows into a binding before the closure so it's not moved
    let row_iter_id = next_extract_id();
    let rows_binding = quote::format_ident!("__quoin_rows_{}", row_iter_id);
    bindings.push(quote! { let #rows_binding = (#rows).clone(); });

    quote! {
        <table class={concat!("w-full", #striped_class)}>
            <thead><tr> #(#header_cells)* </tr></thead>
            <tbody>
                {
                    #rows_binding.clone().into_iter().map(|__row| {
                        leptos::view! { <tr> #(#row_cells)* </tr> }
                    }).collect::<Vec<_>>()
                }
            </tbody>
        </table>
    }
}
