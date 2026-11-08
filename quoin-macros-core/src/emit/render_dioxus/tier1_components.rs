// Emission helpers in this module have `dioxus` and `dioxus-shadcn` variants:
// some imports and helper variables are only exercised by one of the two
// code paths. File-level allows keep every feature combination compiling
// warning-free without per-line noise.

use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

// ---------------------------------------------------------------------------
// Separator
// ---------------------------------------------------------------------------
pub(crate) fn resolve_separator_tag(el: &Element) -> TokenStream {
    let orientation =
        find_arg_string(el, "orientation").unwrap_or_else(|| "horizontal".to_string());
    let _ = &orientation; // read only by one of the two emit paths
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
        if key_str == "class" {
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
        // `orientation` was consumed in `resolve_separator_tag`; the remaining
        // attributes are ignored.
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
    let _ = &el; // read only by the plain-HTML path
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
    let _ = &el; // read only by the plain-HTML path
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
    let _ = &el; // read only by the plain-HTML path
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
    let _ = &outer_cls; // read only by the plain-HTML path

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        let value_prop = match value_expr {
            Some(val) => {
                if let Some(m) = max_expr {
                    quote! { value={format!("{}", (#val as f64) / (#m as f64))} }
                } else {
                    quote! { value={format!("{}", (#val as f64) / 100.0)} }
                }
            }
            None => quote! {},
        };
        quote! { shadcn_dioxus::progress::Progress { #value_prop } }
    }

    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let bar_cls = "h-full rounded-full bg-primary transition-all duration-300";
        if let Some(val) = value_expr {
            let _max = if let Some(m) = max_expr {
                quote! { (#val as f64) / (#m as f64) * 100.0 }
            } else {
                quote! { (#val as f64) }
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
        } else {
            let indeterminate_cls = "h-full w-1/3 rounded-full bg-primary animate-indeterminate";
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
