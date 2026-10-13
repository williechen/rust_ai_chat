use ai_core::ChatModel;
use std::sync::Arc;

pub struct AiState {
    pub model: Arc<dyn ChatModel>,
}

impl AiState {
    pub fn new(model: Arc<dyn ChatModel>) -> Self {
        Self { model }
    }
}
