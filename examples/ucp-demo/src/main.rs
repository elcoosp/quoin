use gpui::*;
use gpui_platform::application;
use quoin_gpui::GpuiContext;
use ucp_lib::DemoApp;

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx
            .open_window(WindowOptions::default(), |window, window_cx| {
                window_cx.new(|cx: &mut Context<DemoApp>| {
                    let ctx: GpuiContext = cx.into();
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                    DemoApp::new(cx, ctx, DemoAppProps {})
                })
            })
            .unwrap();
        app_cx.activate(true);
    });
}
