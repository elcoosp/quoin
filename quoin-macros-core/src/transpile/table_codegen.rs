//! Data table code generation for framework-specific table components.
//!
//! Provides [`ColumnDef`] as a shared column descriptor, and per-framework
//! generators:
//!
//! - **GPUI** ([`generate_gpui_table_delegate`]): Generates a struct implementing
//!   `gpui_component::table::TableDelegate` with `row_count`, `column_count`,
//!   `perform_sort`, and `render_td` methods. Each column's render closure is
//!   stored as an `Arc<dyn Fn>`. Sortable columns map their index to a string key
//!   for the `on_sort` callback.
//!
//! - **Leptos** ([`generate_leptos_table`]): Generates a `<table>` using
//!   `leptos_shadcn_ui::table` components (or falls back to plain HTML `<table>`
//!   if shadcn is unavailable). Uses `For` component for row iteration.
//!
//! - **Dioxus** ([`generate_dioxus_table`]): Generates an `rsx!` table with
//!   `thead`/`tbody` and iterator-based row rendering.
//!
//! > **Note:** The GPUI delegate generator is not currently used by the inline
//! > `data_table` element in `render_gpui.rs` (which emits a simpler manual
//! > layout). It is reserved for future integration with `gpui_component::table`.

#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;
pub struct ColumnDef {
    pub key: String,
    pub label: String,
    pub width: Option<f32>,
    pub sortable: bool,
    pub render_closure: syn::Expr,
}

#[cfg(feature = "gpui")]
pub fn generate_gpui_table_delegate(
    delegate_name: &proc_macro2::Ident,
    row_type: &syn::Type,
    columns: &[ColumnDef],
) -> TokenStream {
    let col_count = columns.len();

    let field_defs: Vec<TokenStream> = columns
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let fname =
                proc_macro2::Ident::new(&format!("__col_{}", i), proc_macro2::Span::call_site());
            quote! { #fname: std::sync::Arc<dyn Fn(&#row_type) -> gpui::AnyElement + Send + Sync> }
        })
        .collect();

    let field_names: Vec<proc_macro2::Ident> = columns
        .iter()
        .enumerate()
        .map(|(i, _)| {
            proc_macro2::Ident::new(&format!("__col_{}", i), proc_macro2::Span::call_site())
        })
        .collect();

    let match_arms: Vec<TokenStream> = columns
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let idx = i;
            let fname = &field_names[i];
            quote! { #idx => (self.#fname)(row).into_any_element() }
        })
        .collect();

    let sort_arms: Vec<TokenStream> = columns
        .iter()
        .enumerate()
        .filter(|(_, col)| col.sortable)
        .map(|(i, col)| {
            let idx = i;
            let key = &col.key;
            quote! { #idx => #key.to_string() }
        })
        .collect();

    let sort_arms_fallback = if sort_arms.is_empty() {
        quote! { _ => return }
    } else {
        quote! { _ => return }
    };

    quote! {
        struct #delegate_name {
            rows: Vec<#row_type>,
            on_sort: std::sync::Arc<dyn Fn(String, quoin_ui::SortDirection) + Send + Sync>,
            #(#field_defs),*
        }

        impl gpui_component::table::TableDelegate for #delegate_name {
            fn row_count(&self, _: &mut gpui::App) -> usize {
                self.rows.len()
            }

            fn column_count(&self, _: &mut gpui::App) -> usize {
                #col_count
            }

            fn perform_sort(
                &mut self,
                col_ix: usize,
                sort: gpui_component::table::ColumnSort,
                _: &mut gpui::Window,
                _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) {
                let key = match col_ix {
                    #(#sort_arms,)*
                    #sort_arms_fallback
                };
                let dir = match sort {
                    gpui_component::table::ColumnSort::Ascending => quoin_ui::SortDirection::Asc,
                    gpui_component::table::ColumnSort::Descending => quoin_ui::SortDirection::Desc,
                    _ => return,
                };
                (self.on_sort)(key, dir);
            }

            fn render_td(
                &mut self,
                row_ix: usize,
                col_ix: usize,
                _: &mut gpui::Window,
                _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
            ) -> impl gpui::IntoElement {
                let row = &self.rows[row_ix];
                match col_ix {
                    #(#match_arms,)*
                    _ => gpui::div().into_any_element(),
                }
            }
        }
    }
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_table(
    row_type: &syn::Type,
    columns: &[ColumnDef],
    rows_expr: &syn::Expr,
    striped: bool,
) -> TokenStream {
    let header_cells = columns.iter().map(|col| {
        let label = &col.label;
        let width_attr = if let Some(w) = col.width {
            quote! { style:format!("width: {}px", #w) }
        } else {
            quote! {}
        };
        quote! {
            <leptos_shadcn_ui::table::TableHead class=#width_attr>
                #label
            </leptos_shadcn_ui::table::TableHead>
        }
    });

    let row_cells = columns.iter().map(|col| {
        let render = &col.render_closure;
        quote! {
            <leptos_shadcn_ui::table::TableCell>
                {#render(row)}
            </leptos_shadcn_ui::table::TableCell>
        }
    });

    let striped_class = if striped { "table-striped" } else { "" };

    quote! {
        <leptos_shadcn_ui::table::Table class=#striped_class>
            <leptos_shadcn_ui::table::TableHeader>
                <leptos_shadcn_ui::table::TableRow>
                    #(#header_cells)*
                </leptos_shadcn_ui::table::TableRow>
            </leptos_shadcn_ui::table::TableHeader>
            <leptos_shadcn_ui::table::TableBody>
                <leptos::prelude::For
                    each=move || #rows_expr.clone()
                    key=|row| row.id
                    children=move |row: #row_type| {
                        leptos::prelude::view! {
                            <leptos_shadcn_ui::table::TableRow>
                                #(#row_cells)*
                            </leptos_shadcn_ui::table::TableRow>
                        }
                    }
                />
            </leptos_shadcn_ui::table::TableBody>
        </leptos_shadcn_ui::table::Table>
    }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_table(
    _row_type: &syn::Type,
    columns: &[ColumnDef],
    rows_expr: &syn::Expr,
    striped: bool,
) -> TokenStream {
    let header_cells = columns.iter().map(|col| {
        let label = &col.label;
        quote! {
            th { #label }
        }
    });

    let row_cells = columns.iter().map(|col| {
        let render = &col.render_closure;
        quote! {
            td { #render(row) }
        }
    });

    let striped_attr = if striped {
        quote! { striped: true }
    } else {
        quote! {}
    };

    quote! {
        table { #striped_attr
            thead {
                tr { #(#header_cells)* }
            }
            tbody {
                #rows_expr.iter().map(|row| {
                    rsx! { tr { #(#row_cells)* } }
                })
            }
        }
    }
}
