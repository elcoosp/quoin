# quoin-gpui

[![Crates.io](https://img.shields.io/crates/v/quoin-gpui.svg)](https://crates.io/crates/quoin-gpui)
[![Docs.rs](https://docs.rs/quoin-gpui/badge.svg)](https://docs.rs/quoin-gpui)

GPUI adapter for [quoin](https://crates.io/crates/quoin).

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
quoin = "0.1"
quoin-gpui = "0.1"
```

### Basic Counter Example

```rust
use gpui::*;
use gpui_platform::application;
use quoin::ReactiveContext;
use quoin_gpui::GpuiContext;

struct CounterView {
    count: quoin_gpui::GpuiSignal<u32>,
    _ctx: GpuiContext,
}

impl Render for CounterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e2e2e))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Count: {}", self.count.get()))
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x4e4e4e))
                    .rounded_md()
                    .cursor_pointer()
                    .child("Increment")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _ev, _window, _cx| {
                        this.count.update(|c| *c += 1);
                    })),
            )
    }
}

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx
            .open_window(WindowOptions::default(), |window, window_cx| {
                window_cx.new(|cx: &mut Context<CounterView>| {
                    // Create the reactive context from the GPUI context
                    let ctx: GpuiContext = cx.into();

                    // Wire automatic view updates when any signal changes
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));

                    // Create signals using the context
                    let count = ctx.create_signal(0u32);

                    CounterView { count, _ctx: ctx }
                })
            })
            .unwrap();
        app_cx.activate(true);
    });
}
```

### Key Integration Points

1. **Create a `GpuiContext`** using `cx.into()` (or `GpuiContext::new(cx)`).

2. **Wire automatic view updates** by calling `set_view_update_notifier`. This ensures the view repaints whenever any signal created from this context changes.

3. **Create signals** via `ctx.create_signal(initial_value)`.

4. **Store the context** in your view struct (or pass it to components) to keep it alive.

## Conformance

This adapter passes the `quoin-conformance` test suite, guaranteeing identical reactive behavior across all supported frameworks.

## License

MIT OR Apache-2.0
