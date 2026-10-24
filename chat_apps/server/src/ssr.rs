use crate::state::AppState;
use axum::{body::Body, extract::State, http::Request, response::IntoResponse};
use leptos_axum::render_app_to_stream;

pub async fn app_handler(
    State(state): State<AppState>,
    request: Request<Body>,
) -> impl IntoResponse {
    let options = state.leptos_options.clone();

    let handler = render_app_to_stream(move || ui::shell(options.clone()));

    handler(request).await.into_response()
}
