use crate::emit::common::find_arg_expr;
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn emit_data_table(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let rows = find_arg_expr(el, "rows").unwrap();

        let header_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let label = find_arg_expr(e, "label").unwrap();
                    Some(quote! { th { class: "px-3 py-2 text-gray-400 font-medium", #label } })
                } else {
                    None
                }
            })
            .collect();

        let row_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let render_closure = find_arg_expr(e, "render").unwrap();
                    Some(quote! { td { class: "px-3 py-2 text-white", { (#render_closure)(&__row) } } })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            table { class: "w-full text-sm",
                thead { tr { #(#header_cells)* } }
                tbody {
                    for __row in #rows {
                        tr { #(#row_cells)* }
                    }
                }
            }
        }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let rows = find_arg_expr(el, "rows").unwrap();

        let header_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let label = find_arg_expr(e, "label").unwrap();
                    Some(quote! { th { #label } })
                } else {
                    None
                }
            })
            .collect();

        let row_cells: Vec<TokenStream> = el
            .children
            .iter()
            .filter_map(|c| {
                if let RenderNode::Element(e) = c
                    && e.name == "column"
                {
                    let render_closure = find_arg_expr(e, "render").unwrap();
                    Some(quote! { td { { (#render_closure)(&__row) } } })
                } else {
                    None
                }
            })
            .collect();

        quote! {
            table {
                thead { tr { #(#header_cells)* } }
                tbody {
                    for __row in #rows {
                        tr { #(#row_cells)* }
                    }
                }
            }
        }
    }
}
