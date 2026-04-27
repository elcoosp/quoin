//! Data table code generation for framework-specific table components.
//!
//! Provides [`ColumnDef`] as a shared column descriptor, and per-framework
//! generators.
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

    let sort_arms_fallback = quote! { _ => return };

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
