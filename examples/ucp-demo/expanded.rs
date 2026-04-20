    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling gpui_macos v0.1.0 (https://github.com/zed-industries/zed#5a589c4b)
warning: missing documentation for a module
   --> quoin/src/lib.rs:104:1
    |
104 | pub mod macros;
    | ^^^^^^^^^^^^^^
    |
note: the lint level is defined here
   --> quoin/src/lib.rs:87:9
    |
 87 | #![warn(missing_docs, clippy::all, clippy::pedantic)]
    |         ^^^^^^^^^^^^
    Checking quoin-gpui v0.1.0 (/Users/adm/Documents/Repos/quoin/quoin-gpui)
   Compiling quoin-macros v0.1.0 (/Users/adm/Documents/Repos/quoin/quoin-macros)
warning: unused variable: `rows_expr`
   --> quoin-macros/src/emit/render_gpui.rs:102:9
    |
102 |     let rows_expr = find_arg_expr(el, "rows").expect("dat...
    |         ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_rows_expr`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default
warning: unused variable: `row_type`
   --> quoin-macros/src/transpile/table_codegen.rs:159:5
    |
159 |     row_type: &syn::Type,
    |     ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_row_type`
warning: unused variable: `text_expr`
  --> quoin-macros/src/transpile/rich_text_codegen.rs:75:5
   |
75 |     text_expr: &syn::Expr,
   |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_text_expr`
warning: type `PropDef` is more private than the item `CustomElementDef::props`
  --> quoin-macros/src/custom_element.rs:8:5
   |
 8 |     pub props: Vec<PropDef>,
   |     ^^^^^^^^^^^^^^^^^^^^^^^ field `CustomElementDef::props` is reachable at visibility `pub(crate)`
   |
note: but type `PropDef` is only usable at visibility `pub(self)`
  --> quoin-macros/src/custom_element.rs:11:1
   |
11 | struct PropDef {
   | ^^^^^^^^^^^^^^
   = note: `#[warn(private_interfaces)]` on by default
warning: field `default` is never read
  --> quoin-macros/src/parse.rs:17:9
   |
14 | pub struct PropField {
   |            --------- field in this struct
...
17 |     pub default: Option<Expr>,
   |         ^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
warning: function `emit_component` is never used
 --> quoin-macros/src/emit/leptos.rs:5:8
  |
5 | pub fn emit_component(ast: &ComponentAst) -> TokenStream {
  |        ^^^^^^^^^^^^^^
warning: function `emit_component` is never used
 --> quoin-macros/src/emit/dioxus.rs:5:8
  |
5 | pub fn emit_component(ast: &ComponentAst) -> TokenStream {
  |        ^^^^^^^^^^^^^^
warning: function `emit_render` is never used
  --> quoin-macros/src/emit/render_leptos.rs:11:8
   |
11 | pub fn emit_render(node: &RenderNode) -> TokenStream {
   |        ^^^^^^^^^^^
warning: function `emit_element` is never used
  --> quoin-macros/src/emit/render_leptos.rs:21:4
   |
21 | fn emit_element(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^
warning: function `emit_builtin` is never used
  --> quoin-macros/src/emit/render_leptos.rs:34:4
   |
34 | fn emit_builtin(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^
warning: function `emit_virtual_list` is never used
  --> quoin-macros/src/emit/render_leptos.rs:59:4
   |
59 | fn emit_virtual_list(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^^^^
warning: function `emit_rich_text` is never used
  --> quoin-macros/src/emit/render_leptos.rs:67:4
   |
67 | fn emit_rich_text(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^
warning: function `emit_dropdown` is never used
  --> quoin-macros/src/emit/render_leptos.rs:75:4
   |
75 | fn emit_dropdown(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^
warning: function `emit_tabs` is never used
  --> quoin-macros/src/emit/render_leptos.rs:90:4
   |
90 | fn emit_tabs(el: &Element) -> TokenStream {
   |    ^^^^^^^^^
warning: function `emit_data_table` is never used
  --> quoin-macros/src/emit/render_leptos.rs:95:4
   |
95 | fn emit_data_table(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^^
warning: function `emit_if` is never used
   --> quoin-macros/src/emit/render_leptos.rs:115:4
    |
115 | fn emit_if(if_node: &IfNode) -> TokenStream {
    |    ^^^^^^^
warning: function `emit_for_each` is never used
   --> quoin-macros/src/emit/render_leptos.rs:126:4
    |
126 | fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    |    ^^^^^^^^^^^^^
warning: function `emit_nodes` is never used
   --> quoin-macros/src/emit/render_leptos.rs:133:4
    |
133 | fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    |    ^^^^^^^^^^
warning: function `find_arg_expr` is never used
   --> quoin-macros/src/emit/render_leptos.rs:138:4
    |
138 | fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Optio...
    |    ^^^^^^^^^^^^^
warning: function `find_arg_lit_string` is never used
   --> quoin-macros/src/emit/render_leptos.rs:141:4
    |
141 | fn find_arg_lit_string(el: &Element, key: &str) -> Option...
    |    ^^^^^^^^^^^^^^^^^^^
warning: function `emit_render` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:11:8
   |
11 | pub fn emit_render(node: &RenderNode) -> TokenStream {
   |        ^^^^^^^^^^^
warning: function `emit_element` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:21:4
   |
21 | fn emit_element(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^
warning: function `emit_builtin` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:34:4
   |
34 | fn emit_builtin(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^
warning: function `emit_virtual_list` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:53:4
   |
53 | fn emit_virtual_list(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^^^^
warning: function `emit_rich_text` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:61:4
   |
61 | fn emit_rich_text(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^
warning: function `emit_dropdown` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:69:4
   |
69 | fn emit_dropdown(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^
warning: function `emit_tabs` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:84:4
   |
84 | fn emit_tabs(_el: &Element) -> TokenStream { quote! { div ...
   |    ^^^^^^^^^
warning: function `emit_data_table` is never used
  --> quoin-macros/src/emit/render_dioxus.rs:86:4
   |
86 | fn emit_data_table(el: &Element) -> TokenStream {
   |    ^^^^^^^^^^^^^^^
warning: function `emit_if` is never used
   --> quoin-macros/src/emit/render_dioxus.rs:106:4
    |
106 | fn emit_if(if_node: &IfNode) -> TokenStream {
    |    ^^^^^^^
warning: function `emit_for_each` is never used
   --> quoin-macros/src/emit/render_dioxus.rs:117:4
    |
117 | fn emit_for_each(fe: &ForEachNode) -> TokenStream {
    |    ^^^^^^^^^^^^^
warning: function `emit_nodes` is never used
   --> quoin-macros/src/emit/render_dioxus.rs:124:4
    |
124 | fn emit_nodes(nodes: &[RenderNode]) -> TokenStream {
    |    ^^^^^^^^^^
warning: function `find_arg_expr` is never used
   --> quoin-macros/src/emit/render_dioxus.rs:129:4
    |
129 | fn find_arg_expr<'a>(el: &'a Element, key: &str) -> Optio...
    |    ^^^^^^^^^^^^^
warning: function `find_arg_lit_string` is never used
   --> quoin-macros/src/emit/render_dioxus.rs:132:4
    |
132 | fn find_arg_lit_string(el: &Element, key: &str) -> Option...
    |    ^^^^^^^^^^^^^^^^^^^
warning: fields `label`, `width`, and `render_closure` are never read
  --> quoin-macros/src/transpile/table_codegen.rs:8:9
   |
 6 | pub struct ColumnDef {
   |            --------- fields in this struct
 7 |     pub key: String,
 8 |     pub label: String,
   |         ^^^^^
 9 |     pub width: Option<f32>,
   |         ^^^^^
10 |     pub sortable: bool,
11 |     pub render_closure: syn::Expr,
   |         ^^^^^^^^^^^^^^
warning: function `generate_leptos_table` is never used
   --> quoin-macros/src/transpile/table_codegen.rs:103:8
    |
103 | pub fn generate_leptos_table(
    |        ^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_dioxus_table` is never used
   --> quoin-macros/src/transpile/table_codegen.rs:158:8
    |
158 | pub fn generate_dioxus_table(
    |        ^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_leptos_virtual_list` is never used
  --> quoin-macros/src/transpile/virtual_list_codegen.rs:36:8
   |
36 | pub fn generate_leptos_virtual_list(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_dioxus_virtual_list` is never used
  --> quoin-macros/src/transpile/virtual_list_codegen.rs:52:8
   |
52 | pub fn generate_dioxus_virtual_list(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_leptos_rich_text` is never used
  --> quoin-macros/src/transpile/rich_text_codegen.rs:41:8
   |
41 | pub fn generate_leptos_rich_text(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_dioxus_rich_text` is never used
  --> quoin-macros/src/transpile/rich_text_codegen.rs:74:8
   |
74 | pub fn generate_dioxus_rich_text(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_leptos_dropdown` is never used
  --> quoin-macros/src/transpile/dropdown_codegen.rs:36:8
   |
36 | pub fn generate_leptos_dropdown(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^
warning: function `generate_dioxus_dropdown` is never used
  --> quoin-macros/src/transpile/dropdown_codegen.rs:61:8
   |
61 | pub fn generate_dioxus_dropdown(
   |        ^^^^^^^^^^^^^^^^^^^^^^^^
    Checking gpui_platform v0.1.0 (https://github.com/zed-industries/zed#5a589c4b)
    Checking ucp-demo v0.1.0 (/Users/adm/Documents/Repos/quoin/examples/ucp-demo)
error: expected `,`, or `}`, found `_ctx`
  --> examples/ucp-demo/src/main.rs:78:2
   |
78 | }
   |  ^ help: try adding a comma: `,`
   |
   = note: this error originates in the macro `component` (in Nightly builds, run with -Z macro-backtrace for more info)
error: expected one of `,`, `:`, or `}`, found `_ctx`
  --> examples/ucp-demo/src/main.rs:8:1
   |
 8 | / component! {
 9 | |     DemoApp {
10 | |         state {
11 | |             count: u32 = 0,
12 | |             selected: String = "Option A".to_string(),
13 | |             rows: Vec<Person> = vec![
   | |             ---- while parsing this struct field
...  |
78 | | }
   | | ^
   | | |
   | |_expected one of `,`, `:`, or `}`
   |   while parsing this struct
   |
   = note: this error originates in the macro `component` (in Nightly builds, run with -Z macro-backtrace for more info)

#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use gpui::*;
use gpui_platform::application;
use quoin::ReactiveContext;
use quoin::Signal;
use quoin_gpui::GpuiContext;
use quoin_macros::component;
pub struct DemoAppProps {}
#[automatically_derived]
impl ::core::clone::Clone for DemoAppProps {
    #[inline]
    fn clone(&self) -> DemoAppProps {
        DemoAppProps {}
    }
}
pub struct DemoApp {
    props: DemoAppProps,
    count: quoin_gpui::GpuiSignal<u32>,
    selected: quoin_gpui::GpuiSignal<String>,
    rows: quoin_gpui::GpuiSignal<Vec<Person>>,
    _ctx: quoin_gpui::GpuiContext,
}
impl DemoApp {
    pub fn new(cx: &mut gpui::Context<Self>, props: DemoAppProps) -> Self {
        let ctx: quoin_gpui::GpuiContext = cx.into();
        let count = ctx.create_signal(0);
        let selected = ctx.create_signal("Option A".to_string());
        let rows = ctx
            .create_signal(
                <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        Person {
                            id: 1,
                            name: "Alice".to_string(),
                            age: 30,
                        },
                        Person {
                            id: 2,
                            name: "Bob".to_string(),
                            age: 25,
                        },
                    ]),
                ),
            );
        Self { props, count, selected }
    }
    fn increment(count: quoin_gpui::GpuiSignal<u32>) {
        count.update(|c| *c += 1);
    }
    fn select_option(selected: quoin_gpui::GpuiSignal<String>, option: String) {
        selected.set(option);
    }
}
impl gpui::Render for DemoApp {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .child(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Count: {0}", self.count.get()),
                            )
                        }),
                    ),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x4e4e4e))
                    .rounded_md()
                    .cursor_pointer()
                    .child("Increment")
                    .on_mouse_down(
                        MouseButton::Left,
                        {
                            let count = self.count.clone();
                            move |_ev, _window, _cx| {
                                Self::increment(count.clone());
                            }
                        },
                    ),
            )
            .child(
                div()
                    .child(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Selected: {0}", self.selected.get()),
                            )
                        }),
                    ),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x2563eb))
                    .rounded_md()
                    .cursor_pointer()
                    .child("Select Option A")
                    .on_mouse_down(
                        MouseButton::Left,
                        {
                            let selected = self.selected.clone();
                            move |_ev, _window, _cx| {
                                Self::select_option(
                                    selected.clone(),
                                    "Option A".to_string(),
                                );
                            }
                        },
                    ),
            )
            .child(div().child("People:"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(
                        self
                            .rows
                            .get()
                            .iter()
                            .map(|person| {
                                div()
                                    .child(
                                        ::alloc::__export::must_use({
                                            ::alloc::fmt::format(
                                                format_args!("{0} - {1}", person.name, person.age),
                                            )
                                        }),
                                    )
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
    }
}
struct Person {
    id: u32,
    name: String,
    age: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for Person {
    #[inline]
    fn clone(&self) -> Person {
        Person {
            id: ::core::clone::Clone::clone(&self.id),
            name: ::core::clone::Clone::clone(&self.name),
            age: ::core::clone::Clone::clone(&self.age),
        }
    }
}
fn main() {
    application()
        .run(|app_cx: &mut App| {
            app_cx
                .open_window(
                    WindowOptions::default(),
                    |window, window_cx| {
                        window_cx
                            .new(|cx: &mut Context<DemoApp>| {
                                let ctx: GpuiContext = cx.into();
                                ctx.set_view_update_notifier(
                                    cx.weak_entity(),
                                    window.to_async(cx),
                                );
                                DemoApp::new(cx, DemoAppProps {})
                            })
                    },
                )
                .unwrap();
            app_cx.activate(true);
        });
}
