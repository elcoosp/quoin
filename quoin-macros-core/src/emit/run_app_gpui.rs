use crate::run_app::RunAppInput;
use proc_macro2::TokenStream;
use quote::quote;

pub fn emit_run_app(input: &RunAppInput) -> TokenStream {
    let component = &input.component;
    let window_opts = match &input.window_opts {
        Some(expr) => quote! { #expr },
        None => quote! { gpui::WindowOptions::default() },
    };

    quote! {
        use gpui::AppContext;

        fn main() {
            quoin::launch(|app_cx: &mut gpui::App| {
                app_cx
                    .open_window(#window_opts, |window, window_cx| {
                        window_cx.new(|cx: &mut gpui::Context<#component>| {
                            let ctx: quoin::GpuiContext = cx.into();
                            ctx.set_view_update_notifier(
                                cx.weak_entity(),
                                window.to_async(cx),
                            );
                            #component::new(cx, ctx, Default::default())
                        })
                    })
                    .unwrap();
                app_cx.activate(true);
            });
        }
    }
}
