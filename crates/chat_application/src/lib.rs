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

pub async fn persist_in_existing_room(
    rooms: &dyn RoomRepository,
    messages: &dyn MessageRepository,
    message: ChatMessage,
    publish: impl FnOnce(ChatMessage),
) -> Result<(), RepositoryError> {
    if rooms.find_by_id(&message.room_id).await?.is_none() {
        return Err(RepositoryError::RoomAccessError("房間不存在".into()));
    }
    persist_then_publish(messages, message, publish).await
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

    struct SuccessfulRepository;
#[async_trait]
impl MessageRepository for SuccessfulRepository {
    async fn create(&self, _: &ChatMessage) -> Result<(), RepositoryError> {
        Ok(())
    }
}

struct ExistingRoom;
#[async_trait]
impl RoomRepository for ExistingRoom {
    async fn find_by_id(&self, id: &str) -> Result<Option<Room>, RepositoryError> {
        Ok(Some(Room {
            id: id.into(),
            category_room_id: "c1".into(),
            name: "room".into(),
        }))
    }
    async fn create(&self, _: &Room) -> Result<(), RepositoryError> {
        Ok(())
    }
}

struct MissingRoom;
#[async_trait]
impl RoomRepository for MissingRoom {
    async fn find_by_id(&self, _: &str) -> Result<Option<Room>, RepositoryError> {
        Ok(None)
    }
    async fn create(&self, _: &Room) -> Result<(), RepositoryError> {
        Ok(())
    }
}

        struct WrongRoom;
#[async_trait]
impl RoomRepository for WrongRoom {
    async fn find_by_id(&self, _: &str) -> Result<Option<Room>, RepositoryError> {
        Ok(Some(Room {
            id: "another-room".into(),
            category_room_id: "category".into(),
            name: "其他房間".into(),
        }))
    }
    async fn create(&self, _: &Room) -> Result<(), RepositoryError> {
        Ok(())
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

    #[tokio::test]
    async fn missing_room_blocks_message_and_publish() {
        let mut calls = 0;
        let msg = ChatMessage {
            id: "m".into(),
            room_id: "absent".into(),
            author: MessageAuthor::AnonymousUser,
            content: "hi".into(),
        };
        let result =
            persist_in_existing_room(&MissingRoom, &SuccessfulRepository, msg, |_| calls += 1)
                .await;
        assert!(matches!(result, Err(RepositoryError::RoomAccessError(_))));
        assert_eq!(calls, 0);
    }

    #[tokio::test]
    async fn existing_room_allows_persist_and_publish() {
        let mut calls = 0;
        let msg = ChatMessage {
            id: "m".into(),
            room_id: "r1".into(),
            author: MessageAuthor::AnonymousUser,
            content: "hi".into(),
        };
        persist_in_existing_room(&ExistingRoom, &SuccessfulRepository, msg, |_| calls += 1)
            .await
            .unwrap();
        assert_eq!(calls, 1);
    }

#[tokio::test]
async fn mismatched_room_id_must_not_persist_or_publish() {
    let mut published = 0;
    let message = ChatMessage {
        id: "message-1".into(),
        room_id: "requested-room".into(),
        author: MessageAuthor::AnonymousUser,
        content: "hello".into(),
    };
    let result = persist_in_existing_room(
        &WrongRoom,
        &SuccessfulRepository,
        message,
        |_| published += 1,
    ).await;
    assert!(matches!(result, Err(RepositoryError::RoomAccessError(_))));
    assert_eq!(published, 0);
}
}
