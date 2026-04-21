use quoin::Signal;
use quoin_macros::component;

component! {
    TestComponent {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            // GPUI's render block needs to return an element.
            // A simple string literal or div() works perfectly!
            "Hello GPUI"
        }
    }
}

fn main() {}
