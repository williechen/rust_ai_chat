use crate::hub::RoomHub;
use chat_application::ChatService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub servvice_name: &'static str,
    pub chat_service: Arc<ChatService>,
    pub room_hub: RoomHub,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            servvice_name: "rust_ai_chat",
            chat_service: Arc::new(ChatService::new()),
            room_hub: RoomHub::new(128),
        }
    }
}
