use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::{Element, RenderNode};
use crate::transpile::force_move_on_closure;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

/// One `column(...)` child of a `data_table(...)`.
struct ColumnDef {
    key: String,
    label: String,
    width: Option<f32>,
    sortable: bool,
    resizable: bool,
    render: Expr,
}

/// Emits a `data_table(...)` element as a real `gpui_component::DataTable`
/// backed by the shared closure-based [`quoin::QuoinClosureTableDelegate`],
/// with the `TableState` entity persisted across renders through the
/// component's `QuoinTableManager` (field `_quoin_tables`).
///
/// Supported framework-agnostic arguments:
/// - `id: &str` — persistence key (required)
/// - `rows: <Vec<T> expr>` — row data, refreshed on every render
/// - `striped: bool`, `bordered: bool`, `small: bool`
/// - `on_sort: |key: &str, ascending: bool| …` — invoked on column sort
/// - children: `column(key, label, width: f32, sortable: bool, resizable: bool,
///   render: |row: &T| <IntoElement>)`
pub(crate) fn emit_data_table(el: &Element) -> TokenStream {
    let id = find_arg_string(el, "id").expect("data_table requires 'id:' argument");
    let rows_expr = find_arg_expr(el, "rows").expect("data_table requires 'rows:' argument");
    let striped = find_arg_bool(el, "striped");
    let bordered = find_arg_bool(el, "bordered");
    let small = find_arg_bool(el, "small");
    let on_sort_expr = find_arg_expr(el, "on_sort");

    let columns = collect_columns(el);
    assert!(
        !columns.is_empty(),
        "data_table requires at least one column"
    );
    let row_ty = row_type_of(&columns);

    // Column metadata arrays.
    let col_keys: Vec<&String> = columns.iter().map(|c| &c.key).collect();
    let col_labels: Vec<&String> = columns.iter().map(|c| &c.label).collect();
    let col_widths: Vec<TokenStream> = columns
        .iter()
        .map(|c| {
            c.width
                .map(|w| {
                    let lit = proc_macro2::Literal::f32_suffixed(w);
                    quote! { Some(::gpui::px(#lit)) }
                })
                .unwrap_or(quote! { None })
        })
        .collect();
    let col_sortables: Vec<bool> = columns.iter().map(|c| c.sortable).collect();
    let col_resizables: Vec<bool> = columns.iter().map(|c| c.resizable).collect();

    let renderer_closures = renderer_closures(&columns, &row_ty);

    let on_sort_tokens = on_sort_expr.map_or(quote! { None }, |expr| {
        let with_move = force_move_on_closure(expr);
        quote! {
            Some(::std::sync::Arc::new(#with_move))
        }
    });

    let size_tokens = if small {
        quote! { .with_size(::gpui_component::Size::Small) }
    } else {
        quote! {}
    };

    quote! {
        {
            let __rows: Vec<#row_ty> = #rows_expr;
            let __erased_rows: Vec<::std::sync::Arc<dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync>> =
                __rows
                    .into_iter()
                    .map(|__r| {
                        ::std::sync::Arc::new(__r) as ::std::sync::Arc<
                            dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync,
                        >
                    })
                    .collect();
            if !self._quoin_tables.contains(#id) {
                let __delegate = quoin::QuoinClosureTableDelegate {
                    keys: vec![#(#col_keys),*],
                    labels: vec![#(#col_labels),*],
                    widths: vec![#(#col_widths),*],
                    sortables: vec![#(#col_sortables),*],
                    resizables: vec![#(#col_resizables),*],
                    entries: __erased_rows.clone(),
                    renderers: vec![#(#renderer_closures),*],
                    on_sort: #on_sort_tokens,
                };
                let __entity = cx.new(|__cx| {
                    ::gpui_component::table::TableState::new(__delegate, window, __cx)
                });
                self._quoin_tables.insert(#id, __entity);
            }
            let __entity = self._quoin_tables.get(#id).expect("data table just inserted");
            __entity.update(cx, |__ts, _| {
                let __d = __ts.delegate_mut();
                __d.entries = __erased_rows;
                __d.renderers = vec![#(#renderer_closures),*];
                __d.on_sort = #on_sort_tokens;
            });
            ::gpui_component::table::DataTable::new(&__entity)
                .stripe(#striped)
                .bordered(#bordered)
                #size_tokens
        }
    }
}

/// Collects the `column(...)` children of a `data_table(...)` element.
fn collect_columns(el: &Element) -> Vec<ColumnDef> {
    let mut columns: Vec<ColumnDef> = Vec::new();
    for child in &el.children {
        if let RenderNode::Element(e) = child
            && e.name == "column"
        {
            let render = find_arg_expr(e, "render")
                .expect("column requires 'render:' closure")
                .clone();
            columns.push(ColumnDef {
                key: find_arg_string(e, "key").unwrap_or_default(),
                label: find_arg_string(e, "label").unwrap_or_default(),
                width: find_arg_f32(e, "width"),
                sortable: find_arg_bool(e, "sortable"),
                resizable: find_arg_bool(e, "resizable"),
                render,
            });
        }
    }
    columns
}

/// The row type, taken from the first render closure's `|row: &T|` parameter.
fn row_type_of(columns: &[ColumnDef]) -> syn::Type {
    columns
        .iter()
        .find_map(|col| match &col.render {
            Expr::Closure(closure) => closure.inputs.first().and_then(|input| {
                if let syn::Pat::Type(pat) = input {
                    if let syn::Type::Reference(type_ref) = &*pat.ty {
                        return Some((*type_ref.elem).clone());
                    }
                    return Some((*pat.ty).clone());
                }
                None
            }),
            _ => None,
        })
        .expect("column render closures must be typed: |row: &RowType| …")
}

/// Type-erasing renderer closures per column (`&dyn Any -> AnyElement`).
fn renderer_closures(columns: &[ColumnDef], row_ty: &syn::Type) -> Vec<TokenStream> {
    columns
        .iter()
        .map(|col| {
            let render = &col.render;
            quote! {
                {
                    let __closure = #render;
                    ::std::sync::Arc::new(move |__any: &dyn ::std::any::Any| {
                        let __row = __any
                            .downcast_ref::<#row_ty>()
                            .expect("data_table row type mismatch");
                        ::gpui::IntoElement::into_any_element((__closure)(__row))
                    })
                }
            }
        })
        .collect()
}
