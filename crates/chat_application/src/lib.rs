pub mod product_event;
pub mod repository;

use async_trait::async_trait;
use chat_domain::{ChatMessage, MessageAuthor, Room};
use uuid::Uuid;

use crate::repository::{MessageRepository, RepositoryError, RoomRepository};

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
) -> Result<Room, RepositoryError> {
    let room = Room {
        id: input.id,
        category_room_id: input.category_room_id,
        name: input.name,
    };
    room_repository.create(&room).await?;
    Ok(room)
}

///成功 create 後才推播；DB 失敗時 closure 不執行。
pub async fn persist_then_publish(
    repository: &dyn MessageRepository,
    message: ChatMessage,
    publish: impl FnOnce(ChatMessage),
) -> Result<(), RepositoryError> {
    repository.create(&message).await?;
    publish(message);
    Ok(())
}

struct SuccessfulRepository;
#[async_trait]
impl MessageRepository for SuccessfulRepository {
    async fn create(&self, _: &ChatMessage) -> Result<(), RepositoryError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct FailingRepository;
    #[async_trait]
    impl MessageRepository for FailingRepository {
        async fn create(&self, _: &ChatMessage) -> Result<(), RepositoryError> {
            Err(RepositoryError::MessageAccessError("DB unavailable".into()))
        }
    }
    #[tokio::test]
    async fn failed_write_never_publishes() {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let message = ChatMessage {
            id: "m-1".into(),
            room_id: "r-1".into(),
            author: MessageAuthor::AnonymousUser,
            content: "hello".into(),
        };
        let result = persist_then_publish(&FailingRepository, message, move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn successful_write_publishes_exactly_once() {
        let publishes = Arc::new(AtomicUsize::new(0));
        let observed = publishes.clone();
        let message = ChatMessage {
            id: "m-success".into(),
            room_id: "r-success".into(),
            author: MessageAuthor::AnonymousUser,
            content: "hello".into(),
        };

        persist_then_publish(&SuccessfulRepository, message, move |published| {
            assert_eq!(published.id, "m-success");
            observed.fetch_add(1, Ordering::SeqCst);
        })
        .await
        .expect("successful storage must allow publish");

        assert_eq!(publishes.load(Ordering::SeqCst), 1);
    }
}
