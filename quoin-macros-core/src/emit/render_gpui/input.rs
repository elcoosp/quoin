use crate::emit::common::{find_arg_expr, find_arg_string};
use crate::render_ast::Element;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use super::try_transpile_class;

pub(crate) fn emit_input(el: &Element) -> TokenStream {
    let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();

    let value_expr = match find_arg_expr(el, "value") {
        Some(e) => e,
        None => {
            let mut chain = quote! {
                ::gpui::div().rounded(::gpui::px(6.0)).px(::gpui::px(12.0)).py(::gpui::px(8.0))
                    .bg(::gpui::rgb(0x1f2937)).border_1().border_color(::gpui::rgb(0x374151))
                    .text_color(::gpui::rgb(0x9ca3af)).child(#placeholder)
            };
            if let Some(class_expr) = find_arg_expr(el, "class")
                && let Some(styles) = try_transpile_class(class_expr)
            {
                for style in styles.normal {
                    chain = quote! { #chain #style };
                }
            }
            return chain;
        }
    };

    let value_ident = match value_expr {
        Expr::Path(path) => path.path.segments.last().map(|seg| seg.ident.clone()),
        Expr::Field(field) => match &field.member {
            syn::Member::Named(ident) => Some(ident.clone()),
            _ => None,
        },
        _ => None,
    };

    let value_ident = match value_ident {
        Some(id) => id,
        None => {
            let mut chain = quote! {
                ::gpui::div().rounded(::gpui::px(6.0)).px(::gpui::px(12.0)).py(::gpui::px(8.0))
                    .bg(::gpui::rgb(0x1f2937)).border_1().border_color(::gpui::rgb(0x374151))
                    .text_color(::gpui::rgb(0xffffff)).child(#value_expr)
            };
            if let Some(class_expr) = find_arg_expr(el, "class")
                && let Some(styles) = try_transpile_class(class_expr)
            {
                for style in styles.normal {
                    chain = quote! { #chain #style };
                }
            }
            return chain;
        }
    };

    let input_id_str = format!("__quoin_input_{}", value_ident);
    let has_class = find_arg_expr(el, "class").is_some();
    let mut wrapper_styles = quote! {};
    if let Some(class_expr) = find_arg_expr(el, "class")
        && let Some(styles) = try_transpile_class(class_expr)
    {
        for style in styles.normal {
            wrapper_styles = quote! { #wrapper_styles #style };
        }
    }

    let input_construction = if has_class {
        quote! { ::gpui::div()#wrapper_styles.child(quoin::Input::new(&__entity).appearance(false)) }
    } else {
        quote! { quoin::Input::new(&__entity) }
    };

    quote! {
        {
            let __input_id: &str = #input_id_str;
            if !self._quoin_inputs.contains(__input_id) {
                let __initial_val: String = self.#value_ident.get();
                let __entity = cx.new::<quoin::InputState>(|cx| {
                    let mut __state = quoin::InputState::new(window, cx);
                    __state.set_placeholder(#placeholder, window, cx);
                    __state.set_value(__initial_val, window, cx);
                    __state
                });
                let __sub = cx.observe(&__entity, |this, __entity, cx| {
                    let __new_val: String = __entity.read(cx).value().to_string();
                    this.#value_ident.set(__new_val);
                });
                self._quoin_inputs.insert(__input_id.to_string(), __entity, __sub);
            } else {
                let __entity = self._quoin_inputs.get(__input_id).unwrap();
                let __current: String = __entity.read(cx).value().to_string();
                let __desired: String = self.#value_ident.get();
                if __current != __desired {
                    __entity.update(cx, |__state, cx| { __state.set_value(__desired, window, cx); });
                }
            }
            let __entity = self._quoin_inputs.get(__input_id).unwrap().clone();
            #input_construction
        }
    }
}
