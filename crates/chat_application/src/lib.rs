use chat_domain::{ChatMessage, MessageRole};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct ChatService;

impl ChatService {
    pub fn new() -> Self {
        Self
    }

    pub fn send_message(&self, room_id: Uuid, content: String) -> ChatMessage {
        ChatMessage {
            id: Uuid::new_v4(),
            room_id,
            role: MessageRole::User,
            content,
        }
    }
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
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "hello");
    }
}
