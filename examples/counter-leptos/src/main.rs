#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    use axum::Router;
    use counter_leptos::App;
    use leptos::config::get_configuration;
    use leptos::view;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::ServeDir;

    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, || view! { <App/> })
        .fallback_service(ServeDir::new(leptos_options.site_root.as_ref()))
        .with_state(leptos_options);

    println!("Serving at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("This binary is for SSR only. Use `trunk serve` for WASM.");
}
