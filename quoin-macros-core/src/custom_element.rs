use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token, braced};

/// AST for the `quoin_element!` macro input.
pub struct CustomElementDef {
    /// Element tag name (string literal in the macro input).
    pub name: String,
    /// Declared props.
    pub props: Vec<PropDef>,
    /// Optional render closure `|props| quoin_render! { .. }`.
    pub render_fn: Option<syn::Expr>,
}

/// One prop of a custom element (`name: Type`).
pub struct PropDef {
    /// Prop name.
    name: Ident,
    /// Prop type.
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

        // Parse optional render closure: , |props| quoin_render! { ... }
        let render_fn = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.peek(syn::token::Paren) || input.peek(Token![|]) {
                Some(input.parse::<syn::Expr>()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            name,
            props,
            render_fn,
        })
    }
}

#[must_use]
/// Expands a `quoin_element!` definition into a struct with the declared
/// props plus a render entry point.
pub fn expand_custom_element(def: CustomElementDef) -> TokenStream {
    let name_str = def.name;
    let element_ident = Ident::new(&name_str, proc_macro2::Span::call_site());
    let prop_fields = def.props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        quote! { pub #name: #ty }
    });

    #[cfg(feature = "gpui")]
    let render_impl = match &def.render_fn {
        Some(render_expr) => {
            quote! {
                impl #element_ident {
                    pub fn render<F, E>(&self, _ctx: &F) -> E
                    where
                        F: Clone,
                        E: From<::gpui::Div>,
                    {
                        let _ = _ctx;
                        #render_expr
                    }
                }
            }
        }
        None => {
            quote! {
                impl #element_ident {
                    pub fn render<F, E>(&self, _ctx: &F) -> E
                    where
                        F: Clone,
                        E: From<::gpui::Div>,
                    {
                        let _ = _ctx;
                        ::gpui::div()
                    }
                }
            }
        }
    };

    #[cfg(feature = "leptos")]
    let render_impl = match &def.render_fn {
        Some(render_expr) => {
            quote! {
                impl #element_ident {
                    pub fn render(&self) -> impl leptos::prelude::IntoView {
                        #render_expr
                    }
                }
            }
        }
        None => {
            quote! {
                impl #element_ident {
                    pub fn render(&self) -> impl leptos::prelude::IntoView {
                        leptos::prelude::view! { <div></div> }
                    }
                }
            }
        }
    };

    #[cfg(feature = "dioxus")]
    let render_impl = match &def.render_fn {
        Some(render_expr) => {
            quote! {
                impl #element_ident {
                    pub fn render(&self) -> dioxus::prelude::Element {
                        dioxus::prelude::rsx! { #render_expr }
                    }
                }
            }
        }
        None => {
            quote! {
                impl #element_ident {
                    pub fn render(&self) -> dioxus::prelude::Element {
                        dioxus::prelude::rsx! { div {} }
                    }
                }
            }
        }
    };

    #[cfg(not(any(feature = "gpui", feature = "leptos", feature = "dioxus")))]
    let render_impl = {
        quote! {
            compile_error!("quoin_element! requires a framework feature (gpui, leptos, or dioxus)");
        }
    };

    quote! {
        #[derive(Clone)]
        pub struct #element_ident {
            #(#prop_fields),*
        }

        #render_impl

        #[doc(hidden)]
        pub mod __quoin_elements {
            pub use super::#element_ident as #element_ident;
        }
    }
}

#[must_use]
/// Registry hook for user-registered custom elements; the built-in registry
/// is empty, so this always returns `None` for now.
pub const fn resolve_custom_element(_name: &str) -> Option<TokenStream> {
    None
}
