use async_trait::async_trait;
use chat_domain::{ChatMessage, Room};

use crate::error::{MessageRepositoryError, RoomRepositoryError};

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn find_by_id(&self, room_id: &str) -> Result<Option<Room>, RoomRepositoryError>;

    async fn create(&self, room: &Room) -> Result<(), RoomRepositoryError>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &ChatMessage) -> Result<(), MessageRepositoryError>;
}
