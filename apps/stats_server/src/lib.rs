pub mod pg_store;

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    extract::State,
    http::StatusCode,
    http::{HeaderMap, header::AUTHORIZATION},
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use pg_store::{PersistError, PersistOutcome};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventInput {
    pub schema_version: u16,
    pub event_id: Uuid,
    pub source_app: String,
    pub event_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventIdentity {
    source_app: String,
    event_id: String,
}

impl EventIdentity {
    pub fn new(source_app: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            source_app: source_app.into(),
            event_id: event_id.into(),
        }
    }
}

#[derive(Default)]
pub struct InMemoryDeduplicator {
    seen: std::collections::HashSet<EventIdentity>,
}

impl InMemoryDeduplicator {
    pub fn accept_once(&mut self, identity: EventIdentity) -> bool {
        self.seen.insert(identity)
    }
    /// 無效事件不占用去重鍵；Some(false) 代表本程序已看過。
    /// 僅用於記憶體契約學習，不是資料庫提交收據。
    pub fn accept_valid_event(&mut self, event: &EventInput) -> Option<bool> {
        if !valid_event(event) {
            return None;
        }
        Some(self.accept_once(EventIdentity::new(
            event.source_app.clone(),
            event.event_id.to_string(),
        )))
    }
}

// 事件類型長度以 UTF-8 位元組計算，避免儲存層限制不一致。
pub fn valid_event_type(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.trim() == value
}

// 只做事件資料驗證；憑證、持久化與 ACK 各由真正 Adapter 負責。
pub fn valid_event(event: &EventInput) -> bool {
    event.schema_version == 1
        && !event.event_id.is_nil()
        && matches!(event.source_app.as_str(), "web_chat" | "desktop_pet")
        && valid_event_type(event.event_type.as_str())
}

#[derive(Clone)]
pub struct IngestTokens {
    pub chat: String,
    pub pet: String,
}

#[derive(Clone)]
pub struct IngestState {
    pub pool: PgPool,
    pub tokens: Arc<IngestTokens>,
}

impl IngestTokens {
    fn authorized(&self, source: &str, header: &str) -> bool {
        let token = match source {
            "web_chat" => &self.chat,
            "desktop_pet" => &self.pet,
            _ => return false,
        };
        // 不接受空憑證；固定長度逐位比較避免提早依內容退出。
        if token.is_empty() || token.len() > 512 {
            return false;
        }
        let expected = format!("Bearer {token}");
        let a = expected.as_bytes();
        let b = header.as_bytes();
        let mut diff = a.len() ^ b.len();
        for i in 0..a.len().max(b.len()) {
            diff |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
        }
        diff == 0
    }
}

fn outcome_status(value: PersistOutcome) -> StatusCode {
    match value {
        PersistOutcome::Stored => StatusCode::CREATED,
        PersistOutcome::AlreadyExists => StatusCode::OK,
        PersistOutcome::IdentityConflict => StatusCode::CONFLICT,
    }
}

async fn health() -> StatusCode {
    StatusCode::OK
}

pub async fn ingest(
    State(state): State<IngestState>,
    headers: HeaderMap,
    Json(event): Json<EventInput>,
) -> StatusCode {
    if !valid_event(&event) {
        return StatusCode::BAD_REQUEST;
    }
    let authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !state.tokens.authorized(&event.source_app, authorization) {
        return StatusCode::UNAUTHORIZED;
    }
    match pg_store::persist_event(&state.pool, &event).await {
        Ok(outcome) => outcome_status(outcome),
        Err(PersistError::Invalid) => StatusCode::BAD_REQUEST,
        Err(PersistError::Database(_)) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub fn build_app(state: IngestState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/events", post(ingest))
        .with_state(state)
        .layer(DefaultBodyLimit::max(4096))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pg_store::PersistOutcome::*;

    #[test]
    fn nil_event_id_is_not_accepted() {
        let invalid = EventInput {
            schema_version: 1,
            event_id: Uuid::nil(),
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        assert!(!valid_event(&invalid));
        let valid = EventInput {
            event_id: Uuid::new_v4(),
            ..invalid
        };
        assert!(valid_event(&valid));
    }

    #[test]
    fn rejects_bad_source_version_and_event_type() {
        let mut input = EventInput {
            schema_version: 1,
            event_id: Uuid::new_v4(),
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        assert!(valid_event(&input));
        input.source_app = "invalid_source".into();
        assert!(!valid_event(&input));
        input.source_app = "desktop_pet".into();
        input.schema_version = 2;
        assert!(!valid_event(&input));
        input.schema_version = 1;
        input.event_type = "   ".into();
        assert!(!valid_event(&input));
    }

    #[test]
    fn event_type_rejects_noncanonical_boundaries() {
        assert!(valid_event_type("chat.message.persisted"));
        assert!(!valid_event_type(" chat.message.persisted"));
        assert!(!valid_event_type("chat.message.persisted "));
        assert!(!valid_event_type("\n"));
        assert!(!valid_event_type(""));
        assert!(!valid_event_type(&"x".repeat(129)));
    }

    #[test]
    fn existing_event_validation_rejects_whitespace() {
        let event = EventInput {
            schema_version: 1,
            event_id: Uuid::new_v4(),
            source_app: "web_chat".into(),
            event_type: " chat.message.persisted".into(),
        };
        assert!(!valid_event(&event));
    }

    #[test]
    fn valid_event_is_accepted_once_per_source_and_id() {
        let id = Uuid::new_v4();
        let event = EventInput {
            schema_version: 1,
            event_id: id,
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        let mut dedup = InMemoryDeduplicator::default();
        assert_eq!(dedup.accept_valid_event(&event), Some(true));
        assert_eq!(dedup.accept_valid_event(&event), Some(false));
        let other_source = EventInput {
            source_app: "desktop_pet".into(),
            event_type: "pet.cleanup.succeeded".into(),
            ..event
        };
        assert_eq!(dedup.accept_valid_event(&other_source), Some(true));
    }

    #[test]
    fn invalid_event_does_not_reserve_dedup_key() {
        let id = Uuid::new_v4();
        let mut event = EventInput {
            schema_version: 2,
            event_id: id,
            source_app: "web_chat".into(),
            event_type: "chat.message.persisted".into(),
        };
        let mut dedup = InMemoryDeduplicator::default();
        assert_eq!(dedup.accept_valid_event(&event), None);
        event.schema_version = 1;
        assert_eq!(dedup.accept_valid_event(&event), Some(true));
    }

    #[test]
    fn token_must_match_the_claimed_source() {
        let tokens = IngestTokens {
            chat: "chat-secret".into(),
            pet: "pet-secret".into(),
        };
        assert!(tokens.authorized("web_chat", "Bearer chat-secret"));
        assert!(tokens.authorized("desktop_pet", "Bearer pet-secret"));
        assert!(!tokens.authorized("desktop_pet", "Bearer chat-secret"));
        assert!(!tokens.authorized("web_chat", "Bearer "));
        assert!(!tokens.authorized("web_chat", "chat-secret"));
    }

    #[test]
    fn persist_outcomes_have_distinct_http_codes() {
        assert_eq!(outcome_status(Stored), StatusCode::CREATED);
        assert_eq!(outcome_status(AlreadyExists), StatusCode::OK);
        assert_eq!(outcome_status(IdentityConflict), StatusCode::CONFLICT);
    }
}
