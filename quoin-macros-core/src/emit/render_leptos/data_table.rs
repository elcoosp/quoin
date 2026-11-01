use crate::emit::common::{find_arg_bool, find_arg_expr};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::bindings::next_extract_id;

/// Expands a `data_table` element into Leptos view tokens.
///
/// Under the `leptos-shadcn` feature this emits a `leptos_shadcn_ui::Table`
/// component; otherwise a plain Tailwind-styled `<table>`.
pub(crate) fn emit_data_table(
    el: &Element,
    bindings: &mut Vec<TokenStream>,
    inside_for: bool,
) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    {
        emit_data_table_shadcn(el, bindings)
    }
    #[cfg(not(feature = "leptos-shadcn"))]
    {
        let _ = inside_for;
        emit_data_table_html(el, bindings)
    }
}

/// Expands a `data_table` element into a `leptos_shadcn_ui::Table` component
/// with the declared `column` children mapped to header and body cells
/// (`leptos-shadcn` builds only).
#[cfg(feature = "leptos-shadcn")]
fn emit_data_table_shadcn(el: &Element, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows");
    let striped = find_arg_bool(el, "striped");
    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c
            && e.name == "column"
        {
            let col_label = find_arg_expr(e, "label").unwrap_or(&empty_label);

            header_cells.push(quote! {
                <th class="px-3 py-2 text-gray-400 font-medium">{#col_label}</th>
            });

            let render_closure = find_arg_expr(e, "render");
            if let Some(rc) = render_closure {
                let col_id = next_extract_id();
                let render_name = quote::format_ident!("__quoin_col_{}", col_id);
                bindings.push(quote! { let #render_name = std::sync::Arc::new(#rc); });
                row_cells.push(quote! {
                    <td class="px-3 py-2 text-white">{ (&*#render_name)(&__row) }</td>
                });
            } else {
                row_cells.push(quote! {
                    <td class="px-3 py-2 text-white"></td>
                });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let class_value = if striped {
        "w-full table-striped"
    } else {
        "w-full"
    };

    let table_alias = quote::format_ident!("Table_{}", next_extract_id());
    bindings.push(quote! {
        let #table_alias = leptos_shadcn_ui::Table;
    });

    quote! {
        <#table_alias class=#class_value>
            <thead><tr>#(#header_cells)*</tr></thead>
            <tbody>
                {
                    let __rows = (#rows).clone();
                    __rows.into_iter().map(|__row| {
                        leptos::view! { <tr> #(#row_cells)* </tr> }
                    }).collect::<Vec<_>>()
                }
            </tbody>
        </#table_alias>
    }
}

/// Expands a `data_table` element into a plain Tailwind-styled `<table>` with
/// the declared `column` children mapped to header and body cells (builds
/// without `leptos-shadcn`).
#[cfg(not(feature = "leptos-shadcn"))]
fn emit_data_table_html(el: &Element, bindings: &mut Vec<TokenStream>) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows");
    let striped = find_arg_bool(el, "striped");
    let empty_label: syn::Expr = syn::parse_quote! { "" };
    let mut header_cells = Vec::new();
    let mut row_cells = Vec::new();

    for c in &el.children {
        if let RenderNode::Element(e) = c
            && e.name == "column"
        {
            let col_label = find_arg_expr(e, "label").unwrap_or(&empty_label);
            let width = find_arg_expr(e, "width");
            let mut th_attrs = vec![quote! { class="px-3 py-2 text-gray-400 font-medium" }];
            if let Some(w) = width {
                th_attrs.push(quote! { style=format!("width: {}px", #w) });
            }
            header_cells.push(quote! { <th #(#th_attrs)*>{#col_label}</th> });

            let render_closure = find_arg_expr(e, "render");
            if let Some(rc) = render_closure {
                let col_id = next_extract_id();
                let render_name = quote::format_ident!("__quoin_col_{}", col_id);
                bindings.push(quote! { let #render_name = std::sync::Arc::new(#rc); });
                let mut cell_attrs = vec![quote! { class="px-3 py-2 text-white" }];
                if let Some(w) = width {
                    cell_attrs.push(quote! { style=format!("width: {}px", #w) });
                }
                row_cells.push(quote! { <td #(#cell_attrs)*>{ (&*#render_name)(&__row) }</td> });
            } else {
                row_cells.push(quote! { <td class="px-3 py-2 text-white"></td> });
            }
        }
    }

    let empty_rows: syn::Expr = syn::parse_quote! { Vec::<()>::new() };
    let rows = rows_expr.unwrap_or(&empty_rows);
    let striped_class = if striped { " table-striped" } else { "" };

    quote! {
        <table class={concat!("w-full", #striped_class)}>
            <thead><tr> #(#header_cells)* </tr></thead>
            <tbody>
                {
                    let __rows = (#rows).clone();
                    __rows.into_iter().map(|__row| {
                        leptos::view! { <tr> #(#row_cells)* </tr> }
                    }).collect::<Vec<_>>()
                }
            </tbody>
        </table>
    }
}
