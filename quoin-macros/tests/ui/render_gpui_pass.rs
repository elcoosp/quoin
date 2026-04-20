mod gpui {
    pub type AnyElement = ();
    pub fn div() -> Div { Div }
    pub struct Div;
    impl Div {
        pub fn flex(self) -> Self { self }
        pub fn flex_col(self) -> Self { self }
        pub fn gap_4(self) -> Self { self }
        pub fn p_4(self) -> Self { self }
        pub fn child(self, _: impl IntoElement) -> Self { self }
        pub fn into_any_element(self) -> AnyElement { () }
    }
    pub trait IntoElement {
        fn into_any_element(self) -> AnyElement;
    }
    impl IntoElement for Div {
        fn into_any_element(self) -> AnyElement { () }
    }
    impl IntoElement for &'static str {
        fn into_any_element(self) -> AnyElement { () }
    }
}
use quoin_macros::quoin_render;
fn main() {
    let _ = quoin_render! {
        div(class: "flex flex-col gap-4 p-4") {
            "Hello World"
        }
    };
}
