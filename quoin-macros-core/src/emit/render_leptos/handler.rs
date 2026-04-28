use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn wrap_event_handler(handler_expr: &syn::Expr) -> TokenStream {
    let idents = collect_handler_idents_excluding_params(handler_expr);
    let shadows: Vec<TokenStream> = idents
        .iter()
        .map(|id| quote! { let #id = #id.clone(); })
        .collect();
    let handler_with_move = force_move_on_closure(handler_expr);

    // If the handler is already a block (e.g., `{ let x = ...; move |_| ... }`),
    // inject clones at the beginning of that block to avoid nested blocks
    // that can leak as raw text in the DOM.
    if let syn::Expr::Block(block) = &handler_with_move {
        let stmts = &block.block.stmts;
        quote! {
            {
                #(#shadows)*
                #(#stmts)*
            }
        }
    } else {
        quote! {
            {
                #(#shadows)*
                #handler_with_move
            }
        }
    }
}
