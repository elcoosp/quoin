use quoin_macros::component;

component! {
    TestCounter {
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
