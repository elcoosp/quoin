use gpui::{App, AppContext, Context, WindowOptions}; // ADD THIS LINE
use quoin::prelude::*;
use ucp_lib::MiniDevtools;

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|app_cx: &mut App| {
            gpui_component::init(app_cx);

            app_cx
                .open_window(WindowOptions::default(), |window, window_cx| {
                    let mini_devtools = window_cx.new(|cx: &mut Context<MiniDevtools>| {
                        let ctx: GpuiContext = cx.into();
                        ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                        MiniDevtools::new(cx, ctx, Default::default())
                    });

                    window_cx.new(|cx| gpui_component::Root::new(mini_devtools, window, cx))
                })
                .unwrap();
            app_cx.activate(true);
        });
}
