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

#[derive(Debug, Clone)]
pub struct CreateRoom {
    pub id: String,
    pub category_room_id: String,
    pub name: String,
}

pub async fn create_room(
    room_repository: &dyn RoomRepository,
    input: CreateRoom,
) -> Result<Room, RoomRepositoryError> {
    let room = Room {
        id: input.id,
        category_room_id: input.category_room_id,
        name: input.name,
    };
    room_repository.create(&room).await?;
    Ok(room)
}

#[derive(Debug, Error)]
pub enum RoomRepositoryError {
    #[error("repository error: {0}")]
    Repository(String),
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn find_by_id(&self, room_id: &str) -> Result<Option<Room>, RoomRepositoryError>;

    async fn create(&self, room: &Room) -> Result<(), RoomRepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum MessageRepositoryError {
    #[error("message repository failure: {0}")]
    Repository(String),
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &ChatMessage) -> Result<(), MessageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_message_creates_user_message() {
        let service = ChatService::new();
        let room_id = "rust_chat".to_string();

        let message = service.send_message(room_id.clone(), "hello".to_string());

        assert_eq!(message.room_id, room_id.clone());
        assert_eq!(message.author, MessageAuthor::AnonymousUser);
        assert_eq!(message.content, "hello");
    }
}
