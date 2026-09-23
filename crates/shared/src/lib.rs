use chat_domain::ChatMessage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/*{
  "type": "send_message",
  "data": {
    "room_id": "...",
    "content": "Hello"
  }
}*/
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClientEvent {
    SendMessage { room_id: Uuid, content: String },
    Typing { room_id: Uuid, active: bool },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerEvent {
    MessageCreated(ChatMessage),
    AssistantDelta { message_id: Uuid, delta: String },
    AssistantCompleted { message_id: Uuid },
    PresenceChanged { user_id: Uuid, online: bool },
    Error { code: String, message: String },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_event_json_roundtrip() {
        let event = ClientEvent::SendMessage {
            room_id: Uuid::new_v4(),
            content: String::from("Hello"),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ClientEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
