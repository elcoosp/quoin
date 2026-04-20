use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub struct ColumnDef {
    pub key: String,
    pub label: String,
    pub width: Option<f32>,
    pub sortable: bool,
    pub render_closure: syn::Expr,
}

/// Generate GPUI TableDelegate implementation for gpui-component.
pub fn generate_gpui_table_delegate(
    delegate_name: &Ident,
    row_type: &syn::Type,
    columns: &[ColumnDef],
) -> TokenStream {
    let col_count = columns.len();

    // Fields for each column closure
    let field_defs: Vec<TokenStream> = columns.iter().enumerate().map(|(i, _)| {
        let fname = Ident::new(&format!("__col_{}", i), proc_macro2::Span::call_site());
        quote! { #fname: std::sync::Arc<dyn Fn(&#row_type) -> gpui::AnyElement + Send + Sync> }
    }).collect();

    let field_names: Vec<Ident> = columns.iter().enumerate()
        .map(|(i, _)| Ident::new(&format!("__col_{}", i), proc_macro2::Span::call_site()))
        .collect();

    // Match arms for render_td
    let match_arms: Vec<TokenStream> = columns.iter().enumerate().map(|(i, _)| {
        let idx = i;
        let fname = &field_names[i];
        quote! { #idx => (self.#fname)(row).into_any_element() }
    }).collect();

    // Sort mapping: column index -> key string
    let sort_arms: Vec<TokenStream> = columns.iter().enumerate()
        .filter(|(_, col)| col.sortable)
        .map(|(i, col)| {
            let idx = i;
            let key = &col.key;
            quote! { #idx => #key.to_string() }
        }).collect();

    let sortable_count = columns.iter().filter(|c| c.sortable).count();
    let sort_arms_fallback = if sortable_count == 0 {
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
                cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
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
                cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
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

/// Generate Leptos table using leptos-shadcn-ui Table components.
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

    let row_cells = columns.iter().enumerate().map(|(i, col)| {
        let render = &col.render_closure;
        quote! {
            <leptos_shadcn_ui::table::TableCell>
                {#render(&row)}
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

/// Generate Dioxus table using shadcn-dioxus Table components.
pub fn generate_dioxus_table(
    row_type: &syn::Type,
    columns: &[ColumnDef],
    rows_expr: &syn::Expr,
    striped: bool,
) -> TokenStream {
    let header_cells = columns.iter().map(|col| {
        let label = &col.label;
        quote! {
            shadcn_dioxus::table::TableHead {
                #label
            }
        }
    });

    let row_cells = columns.iter().map(|col| {
        let render = &col.render_closure;
        quote! {
            shadcn_dioxus::table::TableCell {
                #render(row)
            }
        }
    });

    let striped_attr = if striped { quote! { striped: true } } else { quote! {} };

    quote! {
        shadcn_dioxus::table::Table { #striped_attr
            shadcn_dioxus::table::TableHeader {
                shadcn_dioxus::table::TableRow {
                    #(#header_cells)*
                }
            }
            shadcn_dioxus::table::TableBody {
                #rows_expr.iter().map(|row| {
                    rsx! {
                        shadcn_dioxus::table::TableRow {
                            #(#row_cells)*
                        }
                    }
                })
            }
        }
    }
}
