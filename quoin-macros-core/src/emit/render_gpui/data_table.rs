use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use proc_macro2::TokenStream;
use quote::quote;

use super::handler::wrap_handler;

pub(crate) fn emit_data_table(el: &Element) -> TokenStream {
    let rows_expr = find_arg_expr(el, "rows").expect("data_table requires 'rows'");
    let on_sort_expr = find_arg_expr(el, "on_sort");
    let striped = find_arg_bool(el, "striped");

    let header_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let label = find_arg_string(e, "label").unwrap_or_default();
                let key = find_arg_string(e, "key").unwrap_or_default();
                let sortable = find_arg_bool(e, "sortable");
                let width = find_arg_f32(e, "width");

                let mut header = quote! {
                    ::gpui::div()
                        .px(::gpui::px(12.0))
                        .py(::gpui::px(8.0))
                        .text_color(::gpui::rgb(0x6b7280))
                        .font_weight(::gpui::FontWeight::MEDIUM)
                        .child(#label.to_string())
                };

                if let Some(w) = width {
                    header = quote! { #header.w(::gpui::px(#w)) };
                }

                if sortable {
                    if let Some(on_sort) = on_sort_expr {
                        let key_str = key.clone();
                        let wrap = wrap_handler(on_sort);
                        header = quote! {
                            #header
                                .cursor_pointer()
                                .hover(|s| s.bg(::gpui::rgb(0x374151)))
                                .on_mouse_down(::gpui::MouseButton::Left, {
                                    let __handler = ::std::rc::Rc::new(#wrap);
                                    move |_, _, _| { __handler(#key_str, "asc"); }
                                })
                        };
                    } else {
                        header = quote! { #header.cursor_pointer() };
                    }
                }

                return Some(quote! { #header.into_any_element() });
            }
            None
        })
        .collect();

    let row_cells: Vec<TokenStream> = el
        .children
        .iter()
        .filter_map(|c| {
            if let RenderNode::Element(e) = c
                && e.name == "column"
            {
                let render_closure = find_arg_expr(e, "render").expect("column requires 'render'");
                let width = find_arg_f32(e, "width");

                let mut cell = quote! {
                    ::gpui::div()
                        .px(::gpui::px(12.0))
                        .py(::gpui::px(8.0))
                        .text_color(::gpui::rgb(0xffffff))
                        .child((#render_closure)(&__row))
                };

                if let Some(w) = width {
                    cell = quote! { #cell.w(::gpui::px(#w)) };
                }

                return Some(quote! { #cell.into_any_element() });
            }
            None
        })
        .collect();

    let row_renderer = if striped {
        quote! {
            __rows.iter().enumerate().map(|(__i, __row)| {
                let mut __row_el = ::gpui::div().flex().children(vec![#(#row_cells),*]);
                if __i % 2 == 1 { __row_el = __row_el.bg(::gpui::rgb(0x1a1a2e)); }
                __row_el.into_any_element()
            }).collect::<Vec<_>>()
        }
    } else {
        quote! {
            __rows.iter().map(|__row| {
                ::gpui::div().flex().children(vec![#(#row_cells),*]).into_any_element()
            }).collect::<Vec<_>>()
        }
    };

    quote! {
        {
            let __rows = #rows_expr;
            let __header = ::gpui::div().flex().children(vec![#(#header_cells),*]);
            let __row_elements: Vec<::gpui::AnyElement> = #row_renderer;
            ::gpui::div().flex_col().gap_1().size_full().child(__header).children(__row_elements)
        }
    }
}
