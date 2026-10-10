use async_trait::async_trait;
use chat_application::{
    ChatService, MessageRepository, MessageRepositoryError, persist_then_publish,
};
use chat_domain::{ChatMessage, MessageAuthor};
use std::sync::Mutex;

struct FakeRepo {
    fail: bool,
    saved: Mutex<Vec<ChatMessage>>,
}

#[async_trait]
impl MessageRepository for FakeRepo {
    async fn create(&self, message: &ChatMessage) -> Result<(), MessageRepositoryError> {
        if self.fail {
            return Err(MessageRepositoryError::Repository("模擬資料庫失敗".into()));
        }
        self.saved.lock().unwrap().push(message.clone());
        Ok(())
    }
}

fn message() -> ChatMessage {
    ChatMessage {
        id: "m-1".into(),
        room_id: "room-1".into(),
        author: MessageAuthor::AnonymousUser,
        content: "測試訊息".into(),
    }
}

#[tokio::test]
async fn success_persists_before_publish_once() {
    let repo = FakeRepo {
        fail: false,
        saved: Mutex::new(vec![]),
    };
    let published = Mutex::new(Vec::new());
    let input = message();
    persist_then_publish(&repo, input.clone(), |event| {
        assert_eq!(repo.saved.lock().unwrap().len(), 1, "發布前必須先落庫");
        published.lock().unwrap().push(event);
    })
    .await
    .expect("應成功");
    assert_eq!(*repo.saved.lock().unwrap(), vec![input.clone()]);
    assert_eq!(*published.lock().unwrap(), vec![input]);
}

#[tokio::test]
async fn failed_persistence_never_publishes() {
    let repo = FakeRepo {
        fail: true,
        saved: Mutex::new(vec![]),
    };
    let published = Mutex::new(Vec::new());
    let result = persist_then_publish(&repo, message(), |event| {
        published.lock().unwrap().push(event);
    })
    .await;
    assert!(result.is_err());
    assert!(repo.saved.lock().unwrap().is_empty());
    assert!(published.lock().unwrap().is_empty(), "寫入失敗禁止廣播");
}

#[test]
fn send_message_creates_user_message() {
    let service = ChatService::new();
    let room_id = "rust_chat".to_string();

    let message = service.send_message(room_id.clone(), "hello".to_string());

    assert_eq!(message.room_id, room_id.clone());
    assert_eq!(message.author, MessageAuthor::AnonymousUser);
    assert_eq!(message.content, "hello");
}

#[tokio::test]
async fn failed_save_must_never_publish() {
    // Red：Mock repository 的 save() 回傳錯誤。
    // Green：persist_then_publish 先 await save，再呼叫 publish。
    // assert!(result.is_err());
    // assert_eq!(published_count.load(Ordering::SeqCst), 0);
}
