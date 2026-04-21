use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token, braced};

pub struct CustomElementDef {
    pub name: String,
    pub props: Vec<PropDef>,
}

pub struct PropDef {
    name: Ident,
    ty: syn::Type,
}

impl Parse for CustomElementDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_lit: LitStr = input.parse()?;
        let name = name_lit.value();
        input.parse::<Token![,]>()?;
        let props_content;
        braced!(props_content in input);
        let mut props = Vec::new();
        while !props_content.is_empty() {
            let prop_name: Ident = props_content.parse()?;
            props_content.parse::<Token![:]>()?;
            let prop_ty: syn::Type = props_content.parse()?;
            props.push(PropDef {
                name: prop_name,
                ty: prop_ty,
            });
            if !props_content.is_empty() {
                props_content.parse::<Token![,]>()?;
            }
        }
        Ok(CustomElementDef { name, props })
    }
}

pub fn expand_custom_element(def: CustomElementDef) -> TokenStream {
    let name_str = def.name;
    let element_ident = Ident::new(&name_str, proc_macro2::Span::call_site());
    let prop_fields = def.props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        quote! { pub #name: #ty }
    });
    quote! {
        #[derive(Clone)]
        pub struct #element_ident {
            #(#prop_fields),*
        }
        impl #element_ident {
            pub fn render<F>(&self, _ctx: &F) -> impl gpui::IntoElement {
                gpui::div()
            }
        }
        #[doc(hidden)]
        pub mod __quoin_elements {
            pub use super::#element_ident as #element_ident;
        }
    }
}

pub fn resolve_custom_element(_name: &str) -> Option<TokenStream> {
    None
}
