use quoin::component;
use quoin::quoin_render;

component! {
    pub ClipboardTest {
        render {
            quoin_render! {
                div(class: "flex gap-2") {
                    clipboard_button(copy_text: "hello world") { "Copy" }
                }
            }
        }
    }
}

fn main() {}
