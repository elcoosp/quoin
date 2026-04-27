use crate::emit::common::{find_arg_bool, find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;

use super::{emit_render_inner, handler::wrap_dioxus_handler};

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------
pub(crate) fn emit_checkbox(el: &Element) -> TokenStream {
    let checked_expr = find_arg_expr(el, "on_checked_change")
        .or_else(|| find_arg_expr(el, "on_change"))
        .is_some()
        .then(|| find_arg_expr(el, "checked"))
        .flatten();
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    let base = "h-4 w-4 rounded border border-input ring-offset-background accent-primary-500 cursor-pointer";
    let full_class = if user_class.is_empty() {
        base
    } else {
        &user_class
    };

    let checked_attr = match checked_expr {
        Some(val) => quote! { checked: #val, },
        None => quote! {},
    };

    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { onchange: #wrapped, }
        }
        None => quote! {},
    };

    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::checkbox::Checkbox { #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "checkbox", class: #full_class, #checked_attr #on_change_attr #disabled_attr } }
    }
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------
pub(crate) fn emit_switch(el: &Element) -> TokenStream {
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr =
        find_arg_expr(el, "on_checked_change").or_else(|| find_arg_expr(el, "on_change"));
    let disabled = find_arg_bool(el, "disabled");

    let checked_attr = match checked_expr {
        Some(val) => quote! { checked: #val, },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let wrapped = wrap_dioxus_handler(handler);
            quote! { onchange: #wrapped, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::switch::Switch { #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let track_cls = "relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors bg-input";
        let thumb_cls = "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform translate-x-0";
        quote! {
            button {
                class: #track_cls,
                role: "switch",
                #disabled_attr
                onclick: move |_| {},
                input { r#type: "checkbox", #checked_attr #on_change_attr, class: "sr-only peer" },
                div { class: #thumb_cls }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// RadioGroup / Radio
// ---------------------------------------------------------------------------
pub(crate) fn emit_radio_group(el: &Element) -> TokenStream {
    let children: Vec<TokenStream> = el.children.iter().map(|c| emit_render_inner(c)).collect();
    let user_class = find_arg_string(el, "class").unwrap_or_default();

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::radio_group::RadioGroup { class: #user_class, #(#children)* } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        let cls = if user_class.is_empty() {
            "flex flex-col gap-2"
        } else {
            &user_class
        };
        quote! { div { class: #cls, #(#children)* } }
    }
}

pub(crate) fn emit_radio(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let name_expr = find_arg_expr(el, "name");
    let checked_expr = find_arg_expr(el, "checked");
    let on_change_expr = find_arg_expr(el, "on_change");
    let disabled = find_arg_bool(el, "disabled");

    let base = "h-4 w-4 rounded-full border border-input accent-primary-500 cursor-pointer";
    let checked_attr = match checked_expr {
        Some(v) => quote! { checked: #v, },
        None => quote! {},
    };
    let name_attr = match name_expr {
        Some(n) => quote! { name: #n, },
        None => quote! {},
    };
    let value_attr = match value_expr {
        Some(v) => quote! { value: #v },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let w = wrap_dioxus_handler(handler);
            quote! { onchange: #w, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::radio_group::RadioGroupItem { #value_attr #checked_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "radio", class: #base, #checked_attr #name_attr #value_attr #on_change_attr #disabled_attr } }
    }
}

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------
pub(crate) fn emit_slider(el: &Element) -> TokenStream {
    let value_expr = find_arg_expr(el, "value");
    let min_expr = find_arg_expr(el, "min");
    let max_expr = find_arg_expr(el, "max");
    let step_expr = find_arg_expr(el, "step");
    let on_change_expr = find_arg_expr(el, "on_change").or_else(|| find_arg_expr(el, "on_input"));
    let disabled = find_arg_bool(el, "disabled");

    let base =
        "w-full h-2 rounded-lg appearance-none cursor-pointer accent-primary-500 bg-transparent";

    let value_attr = match value_expr {
        Some(v) => quote! { value: #v, },
        None => quote! {},
    };
    let min_attr = match min_expr {
        Some(m) => quote! { min: #m, },
        None => quote! {},
    };
    let max_attr = match max_expr {
        Some(m) => quote! { max: #m, },
        None => quote! {},
    };
    let step_attr = match step_expr {
        Some(s) => quote! { step: #s, },
        None => quote! {},
    };
    let on_change_attr = match on_change_expr {
        Some(handler) => {
            let w = wrap_dioxus_handler(handler);
            quote! { onchange: #w, }
        }
        None => quote! {},
    };
    let disabled_attr = if disabled {
        quote! { disabled: true, }
    } else {
        quote! {}
    };

    #[cfg(all(feature = "dioxus", feature = "dioxus-shadcn"))]
    {
        quote! { shadcn_dioxus::slider::Slider { #value_attr #min_attr #max_attr #step_attr #on_change_attr #disabled_attr } }
    }
    #[cfg(not(all(feature = "dioxus", feature = "dioxus-shadcn")))]
    {
        quote! { input { r#type: "range", class: #base, #value_attr #min_attr #max_attr #step_attr #on_change_attr #disabled_attr } }
    }
}
