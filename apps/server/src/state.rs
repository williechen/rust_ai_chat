use chat_application::ChatService;
use shared::ServerEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct AppState {
    pub servvice_name: &'static str,
    pub chat_service: Arc<ChatService>,
    pub events: broadcast::Sender<ServerEvent>,
}

impl AppState {
    pub fn new() -> Self {
        let (events, _receiver) = broadcast::channel(128);
        Self {
            servvice_name: "rust_ai_chat",
            chat_service: Arc::new(ChatService::new()),
            events,
        }
    }
}
