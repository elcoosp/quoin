use crate::transpile::{collect_handler_idents_excluding_params, force_move_on_closure};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub(crate) fn wrap_handler(handler_expr: &Expr) -> TokenStream {
    let idents = collect_handler_idents_excluding_params(handler_expr);
    let shadows: Vec<TokenStream> = idents
        .iter()
        .map(|id| quote! { let #id = #id.clone(); })
        .collect();
    let handler_with_move = force_move_on_closure(handler_expr);
    quote! {
        {
            #(#shadows)*
            let __handler = ::std::rc::Rc::new(#handler_with_move);
            move |_, _, _| { __handler(()) }
        }
    }
}
