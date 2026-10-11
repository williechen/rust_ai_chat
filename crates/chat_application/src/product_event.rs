use chat_domain::ChatMessage;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedMessageEvent {
    pub schema_version: u16,
    pub event_id: Uuid,
    pub source_app: &'static str,
    pub event_type: &'static str,
}

pub fn persisted_message_event(
    message: &ChatMessage,
) -> Result<PersistedMessageEvent, uuid::Error> {
    Ok(PersistedMessageEvent {
        schema_version: 1,
        event_id: Uuid::parse_str(&message.id)?,
        source_app: "web_chat",
        event_type: "message.persisted",
    })
}
