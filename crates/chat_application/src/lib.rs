use async_trait::async_trait;
use chat_domain::{ChatMessage, MessageAuthor, Room};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ChatService;

impl ChatService {
    pub fn new() -> Self {
        Self
    }

    pub fn send_message(&self, room_id: String, content: String) -> ChatMessage {
        ChatMessage {
            id: Uuid::new_v4().to_string(),
            room_id,
            author: MessageAuthor::AnonymousUser,
            content,
        }
    }
}

#[derive(Debug, Error)]
pub enum RoomRepositoryError {
    #[error("repository error: {0}")]
    Repository(String),
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn find_by_id(&self, room_id: &str) -> Result<Option<Room>, RoomRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_message_creates_user_message() {
        let service = ChatService::new();
        let room_id = Uuid::new_v4();

        let message = service.send_message(room_id, "hello".to_string());

        assert_eq!(message.room_id, room_id);
        assert_eq!(message.author, MessageAuthor::AnonymousUser);
        assert_eq!(message.content, "hello");
    }
}
