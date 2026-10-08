use crate::hub::RoomHub;
use chat_application::{ChatService, RoomRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatService>,
    pub leptos_options: leptos_config::LeptosOptions,
    pub room_hub: RoomHub,
    pub room_repository: Arc<dyn RoomRepository>,
    pub service_name: &'static str,
}

impl AppState {
    pub fn new(
        leptos_options: leptos_config::LeptosOptions,
        room_repository: Arc<dyn RoomRepository>,
    ) -> Self {
        Self {
            service_name: "rust_ai_chat",
            chat_service: Arc::new(ChatService::new()),
            room_hub: RoomHub::new(128),
            leptos_options,
            room_repository,
        }
    }
}
