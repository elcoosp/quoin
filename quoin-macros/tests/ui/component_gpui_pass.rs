// quoin-macros/tests/ui/component_gpui_pass.rs
use gpui::*;
use quoin::ReactiveContext;
use quoin_gpui::GpuiContext;
use quoin_macros::component;

component! {
    TestComponent {
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            div()
        }
    }
}

fn main() {}
