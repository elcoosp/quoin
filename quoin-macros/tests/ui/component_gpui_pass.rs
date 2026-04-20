// Mock the entire quoin_gpui crate
mod quoin_gpui {
    use std::marker::PhantomData;
    use std::rc::Rc;
    use std::cell::RefCell;

    // Required traits from quoin (we can't depend on the real quoin here, so mock)
    pub trait Signal<T: Clone> {
        fn get(&self) -> T;
        fn set(&self, _value: T) {}
        fn update(&self, _f: impl FnOnce(&mut T)) {}
    }

    #[derive(Clone)]
    pub struct GpuiContext;

    impl GpuiContext {
        pub fn create_signal<T: Clone + 'static>(&self, initial: T) -> GpuiSignal<T> {
            GpuiSignal(Rc::new(RefCell::new(initial)))
        }
    }

    #[derive(Clone)]
    pub struct GpuiSignal<T>(Rc<RefCell<T>>);

    impl<T: Clone + 'static> Signal<T> for GpuiSignal<T> {
        fn get(&self) -> T {
            self.0.borrow().clone()
        }
        fn set(&self, value: T) {
            *self.0.borrow_mut() = value;
        }
        fn update(&self, f: impl FnOnce(&mut T)) {
            f(&mut *self.0.borrow_mut());
        }
    }

    // Allow `cx.into()` for creating GpuiContext
    impl<'a, T> From<&'a mut gpui::Context<T>> for GpuiContext {
        fn from(_cx: &'a mut gpui::Context<T>) -> Self {
            Self
        }
    }
}

// Mock the gpui crate
mod gpui {
    pub struct Context<T>(std::marker::PhantomData<T>);
    pub struct Window;
    pub type AnyElement = ();

    pub fn div() -> Div {
        Div
    }
    pub struct Div;
    impl Div {
        pub fn flex(self) -> Self { self }
        pub fn flex_col(self) -> Self { self }
        pub fn bg(self, _: Rgba) -> Self { self }
        pub fn text_color(self, _: Rgba) -> Self { self }
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
    pub trait Render {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement;
    }
    #[derive(Clone, Copy)]
    pub struct Rgba(pub f32, pub f32, pub f32, pub f32);
    pub fn rgb(hex: u32) -> Rgba {
        Rgba(0.0, 0.0, 0.0, 1.0)
    }
    pub type FontWeight = ();
    pub const BOLD: FontWeight = ();
}

use quoin_macros::component;

component! {
    TestComponent {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            gpui::div()
        }
    }
}

fn main() {}
