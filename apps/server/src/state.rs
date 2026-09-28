use crate::hub::RoomHub;
use chat_application::ChatService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub service_name: &'static str,
    pub chat_service: Arc<ChatService>,
    pub room_hub: RoomHub,
    pub leptos_options: leptos_config::LeptosOptions,
}

impl AppState {
    pub fn new(leptos_options: leptos_config::LeptosOptions) -> Self {
        Self {
            service_name: "rust_ai_chat",
            chat_service: Arc::new(ChatService::new()),
            room_hub: RoomHub::new(128),
            leptos_options,
        }
    }
}
