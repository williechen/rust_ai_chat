use crate::state::AppState;
use axum::extract::State;
use axum::{Json, http::StatusCode};
use serde::Serialize;
use shared::{CreateRoomRequest, CreateRoomResponse};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.service_name,
    })
}

pub async fn create_room(
    State(state): State<AppState>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, StatusCode> {
    let room_id = uuid::Uuid::new_v4().to_string();

    let room = chat_application::create_room(
        state.room_repository.as_ref(),
        chat_application::CreateRoom {
            id: room_id,
            category_room_id: "default-category".to_string(),
            name: request.name,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateRoomResponse { id: room.id }))
}
