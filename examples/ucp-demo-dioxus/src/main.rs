// examples/ucp-demo-dioxus/src/main.rs
use dioxus::desktop::Config;
use dioxus::prelude::*;
use ucp_lib::DemoApp;

fn app() -> Element {
    rsx! {
        DemoApp {}
    }
}

fn main() {
    let tailwind_head = r#"
        <script src="https://cdn.tailwindcss.com"></script>
        <style>
            html, body, #main {
                margin: 0;
                padding: 0;
                height: 100%;
                width: 100%;
                overflow: hidden;
            }
        </style>
    "#
    .to_string();

    let cfg = Config::default().with_custom_head(tailwind_head);

    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(app);
}
