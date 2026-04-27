use crate::emit::common::{find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

// ---------------------------------------------------------------------------
// Separator
// ---------------------------------------------------------------------------
pub(crate) fn resolve_separator_tag(el: &Element) -> TokenStream {
    let orientation =
        find_arg_string(el, "orientation").unwrap_or_else(|| "horizontal".to_string());
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::separator::Separator }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let tag = if orientation == "horizontal" {
            "hr"
        } else {
            "div"
        };
        let ident = proc_macro2::Ident::new(tag, proc_macro2::Span::call_site());
        quote! { #ident }
    }
}

pub(crate) fn emit_separator(el: &Element) -> TokenStream {
    let tag = resolve_separator_tag(el);
    let mut attrs: Vec<TokenStream> = Vec::new();
    for arg in &el.args {
        let key_str = arg.key.to_string();
        let value = &arg.value;
        match key_str.as_str() {
            "class" => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = value
                {
                    attrs.push(quote! { class: #s, });
                } else {
                    attrs.push(quote! { class: {#value}, });
                }
            }
            "orientation" => {}
            _ => {}
        }
    }
    if attrs.is_empty() {
        quote! { #tag { } }
    } else {
        quote! { #tag { #(#attrs)* } }
    }
}

// ---------------------------------------------------------------------------
// Skeleton / SkeletonText / SkeletonAvatar
// ---------------------------------------------------------------------------
pub(crate) fn emit_skeleton(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse rounded-md bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse rounded-md bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

pub(crate) fn emit_skeleton_text(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse h-4 w-full rounded-md bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse h-4 w-full rounded-md bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

pub(crate) fn emit_skeleton_avatar(el: &Element) -> TokenStream {
    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::skeleton::Skeleton { class: "animate-pulse h-10 w-10 rounded-full bg-muted" } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let user_class = find_arg_string(el, "class").unwrap_or_default();
        let cls = if user_class.is_empty() {
            "animate-pulse h-10 w-10 rounded-full bg-muted"
        } else {
            &user_class
        };
        quote! { div { class: #cls } }
    }
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------
pub(crate) fn emit_progress(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let max_expr = find_arg_expr(el, "max");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    let outer_cls = if user_class.is_empty() {
        "relative h-4 w-full overflow-hidden rounded-full bg-secondary"
    } else {
        ""
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let value_prop = match value_expr {
            Some(val) => match max_expr {
                Some(m) => quote! { value={format!("{}", (#val as f64) / (#m as f64))} },
                None => quote! { value={format!("{}", (#val as f64) / 100.0)} },
            },
            None => quote! {},
        };
        quote! { shadcn_dioxus::progress::Progress { #value_prop } }
    }

    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let bar_cls = "h-full rounded-full bg-primary transition-all duration-300";
        match value_expr {
            Some(val) => {
                let max = match max_expr {
                    Some(m) => quote! { (#val as f64) / (#m as f64) * 100.0 },
                    None => quote! { (#val as f64) },
                };
                let full_cls = if user_class.is_empty() {
                    outer_cls
                } else {
                    &user_class
                };
                quote! {
                    div { class: #full_cls,
                        div { class: #bar_cls, style: "width: {#max}%" }
                    }
                }
            }
            None => {
                let indeterminate_cls =
                    "h-full w-1/3 rounded-full bg-primary animate-indeterminate";
                let full_cls = if user_class.is_empty() {
                    outer_cls
                } else {
                    &user_class
                };
                quote! {
                    div { class: #full_cls,
                        div { class: #indeterminate_cls }
                    }
                }
            }
        }
    }
}
