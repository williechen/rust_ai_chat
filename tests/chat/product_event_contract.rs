use async_trait::async_trait;
use chat_application::{
    persist_then_publish,
    product_event::persisted_message_event,
    repository::{MessageRepository, RepositoryError},
};
use chat_domain::{ChatMessage, MessageAuthor};
use std::sync::Mutex;
use uuid::Uuid;

struct FakeRepository {
    fail: bool,
    saved: Mutex<Vec<ChatMessage>>,
}

#[async_trait]
impl MessageRepository for FakeRepository {
    async fn create(&self, message: &ChatMessage) -> Result<(), RepositoryError> {
        if self.fail {
            return Err(RepositoryError::MessageAccessError("模擬儲存失敗".into()));
        }
        self.saved.lock().unwrap().push(message.clone());
        Ok(())
    }
}

fn message() -> ChatMessage {
    ChatMessage {
        id: Uuid::from_u128(1).to_string(),
        room_id: "private-room".into(),
        author: MessageAuthor::AnonymousUser,
        content: "不可外送的私人訊息".into(),
    }
}

#[tokio::test]
async fn success_generates_stable_minimal_event_only_after_persist() {
    let repo = FakeRepository {
        fail: false,
        saved: Mutex::new(vec![]),
    };
    let events = Mutex::new(Vec::new());
    let input = message();
    persist_then_publish(&repo, input.clone(), |saved| {
        // 只有成功儲存之後，才允許生成產品事件。
        assert_eq!(repo.saved.lock().unwrap().len(), 1);
        events
            .lock()
            .unwrap()
            .push(persisted_message_event(&saved).unwrap());
    })
    .await
    .unwrap();
    let events = events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_id.to_string(), input.id);
    assert_eq!(events[0].schema_version, 1);
    assert_eq!(events[0].source_app, "web_chat");
    assert_eq!(events[0].event_type, "message.persisted");
}

#[tokio::test]
async fn database_failure_never_generates_event() {
    let repo = FakeRepository {
        fail: true,
        saved: Mutex::new(vec![]),
    };
    let events = Mutex::new(0);
    let result = persist_then_publish(&repo, message(), |saved| {
        if persisted_message_event(&saved).is_ok() {
            *events.lock().unwrap() += 1;
        }
    })
    .await;
    assert!(result.is_err());
    assert_eq!(*events.lock().unwrap(), 0);
}

#[test]
fn same_saved_message_keeps_same_event_id_and_does_not_leak_content() {
    let input = message();
    let a = persisted_message_event(&input).unwrap();
    let b = persisted_message_event(&input).unwrap();
    assert_eq!(a, b);
    let debug = format!("{a:?}");
    assert!(!debug.contains(&input.content));
    assert!(!debug.contains(&input.room_id));
}

#[test]
fn invalid_message_id_is_rejected_not_replaced() {
    let mut input = message();
    input.id = "not-a-uuid".into();
    assert!(persisted_message_event(&input).is_err());
}
