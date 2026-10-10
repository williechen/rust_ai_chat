use async_trait::async_trait;
use chat_application::{
    ChatService, MessageRepository, MessageRepositoryError, persist_then_publish,
};
use chat_domain::{ChatMessage, MessageAuthor};
use std::sync::Mutex;

// 假儲存庫只提供目前三個測試共用的成功／失敗行為。
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

// 測試資料集中建立，避免每個測試重複組裝訊息。
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
        // 驗證發布事件以前，訊息已成功寫入儲存庫。
        assert_eq!(repo.saved.lock().unwrap().len(), 1);
        published.lock().unwrap().push(event);
    })
    .await
    .expect("儲存成功才允許發布");

    // 成功情境只儲存一次、發布一次，事件內容保持一致。
    assert_eq!(*repo.saved.lock().unwrap(), vec![input.clone()]);
    assert_eq!(*published.lock().unwrap(), vec![input]);
}

#[tokio::test]
async fn failed_persistence_never_publishes() {
    let repo = FakeRepo {
        fail: true,
        saved: Mutex::new(vec![]),
    };
    let published = Mutex::new(Vec::<ChatMessage>::new());

    let result = persist_then_publish(&repo, message(), |event| {
        published.lock().unwrap().push(event);
    })
    .await;

    // 合併兩個重複測試的驗證條件：錯誤類型、零儲存、零發布。
    assert!(matches!(
        result,
        Err(MessageRepositoryError::Repository(_))
    ));
    assert!(repo.saved.lock().unwrap().is_empty());
    assert!(published.lock().unwrap().is_empty());
}

#[test]
fn send_message_creates_user_message() {
    let service = ChatService::new();
    let room_id = "rust_chat".to_string();

    let message = service.send_message(room_id.clone(), "hello".to_string());

    assert_eq!(message.room_id, room_id);
    assert_eq!(message.author, MessageAuthor::AnonymousUser);
    assert_eq!(message.content, "hello");
}