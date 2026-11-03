use crate::emit::common::{find_arg_expr, find_arg_f32, find_arg_string};
use crate::render_ast::Element;
use crate::transpile::force_move_on_closure;
use proc_macro2::TokenStream;
use quote::quote;

/// Emits a `virtual_list(...)` element as a real `gpui_component`
/// (gpui-base) `v_virtual_list` with a uniform item size and a
/// manager-persisted scroll handle (field `_quoin_scroll_handles`).
///
/// Supported framework-agnostic arguments:
/// - `id: &str` — element id + scroll-handle persistence key (required)
/// - `count: <usize expr>` — number of items
/// - `item_width: f32` (default 2500.0), `item_height: f32` (default 32.0)
/// - `render: |index: usize| <IntoElement>` — item renderer (re-evaluated
///   per visible item on every frame, so closures may read signals)
pub(crate) fn emit_virtual_list(el: &Element) -> TokenStream {
    let id = find_arg_string(el, "id").expect("virtual_list requires 'id:' argument");
    let count_expr = find_arg_expr(el, "count").expect("virtual_list requires 'count:' argument");
    let item_width = find_arg_f32(el, "item_width").unwrap_or(2500.0);
    let item_height = find_arg_f32(el, "item_height").unwrap_or(32.0);
    // DSL children (`for[item in …] { … }` style bodies) are not supported
    // here; the renderer must be a single expression closure.
    let render_expr = find_arg_expr(el, "render").expect("virtual_list requires 'render:' closure");
    let render_with_move = force_move_on_closure(render_expr);

    let width_lit = proc_macro2::Literal::f32_suffixed(item_width);
    let height_lit = proc_macro2::Literal::f32_suffixed(item_height);

    quote! {
        {
            let __count: usize = #count_expr;
            let __sizes = ::std::rc::Rc::new(vec![
                ::gpui::Size::new(::gpui::px(#width_lit), ::gpui::px(#height_lit));
                __count
            ]);
            let __render = ::std::rc::Rc::new(#render_with_move);
            let __entity = cx.entity();
            let __handle = self._quoin_scroll_handles.virtual_list(#id);
            ::gpui_component::virtual_list::v_virtual_list(
                __entity,
                #id,
                __sizes,
                move |_, __visible_range, _, _| {
                    __visible_range
                        .map(|__ix| (__render)(__ix))
                        .collect::<Vec<_>>()
                },
            )
            .track_scroll(__handle)
        }
    }
}
