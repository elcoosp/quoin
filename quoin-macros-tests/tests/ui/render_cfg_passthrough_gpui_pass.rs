use quoin_macros::{component, quoin_render};

#[derive(Clone)]
pub struct FilteredEvent {
    pub label: String,
}

component! {
    pub CfgTabTest {
        state {
            active_tab: usize = 0,
        }
        render {
            let active = active_tab.get();
            quoin_render! {
                div(class: "flex gap-2") {
                    tabs(active: active, on_click: move |i| active_tab.clone().set(i)) {
                        tab(index: 0, label: "Always")
                        tab(index: 1, label: "GPUI Only")
                        #[cfg(feature = "nexum")]
                        tab(index: 2, label: "Nexum")
                        #[cfg(feature = "fake_feature_xyz")]
                        tab(index: 3, label: "Never Shown")
                    }
                }
            }
        }
    }
}

fn main() {}
