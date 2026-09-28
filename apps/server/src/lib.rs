mod http;
mod hub;
mod ssr;
mod state;
mod ws;

use axum::{
    Router,
    routing::{any, get},
};
use leptos_axum::{site_pkg_dir_service, site_pkg_dir_service_route_path};
use leptos_config::get_configuration;
use std::io::Result;

pub async fn application() -> Result<()> {
    let conf = get_configuration(None).expect("load Leptos Configuration");
    let leptos_options = conf.leptos_options;

    let pkg_path = site_pkg_dir_service_route_path(&leptos_options);
    let pkg_service = site_pkg_dir_service(&leptos_options);

    let state = state::AppState::new(leptos_options);

    let app = Router::new()
        .route_service(&pkg_path, pkg_service)
        .route("/health", get(http::health))
        .route("/ws/{room_id}", any(ws::ws_handler))
        .route("/", get(ssr::app_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
