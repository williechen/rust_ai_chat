use crate::hub::RoomHub;
use chat_application::{ChatService, MessageRepository, RoomRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub chat_service: Arc<ChatService>,
    pub leptos_options: leptos_config::LeptosOptions,
    pub room_hub: RoomHub,
    pub room_repository: Arc<dyn RoomRepository>,
    pub message_repository: Arc<dyn MessageRepository>,
    pub service_name: &'static str,
}

impl AppState {
    pub fn new(
        leptos_options: leptos_config::LeptosOptions,
        room_repository: Arc<dyn RoomRepository>,
        message_repository: Arc<dyn MessageRepository>,
    ) -> Self {
        Self {
            service_name: "rust_ai_chat",
            chat_service: Arc::new(ChatService::new()),
            room_hub: RoomHub::new(128),
            leptos_options,
            room_repository,
            message_repository,
        }
    }
}
