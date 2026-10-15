use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use stats_server::IngestState;
use stats_server::IngestTokens;
use stats_server::{EventInput, ingest, valid_event, valid_event_type};
use uuid::Uuid;

#[test]
fn accepts_normal_kind() {
    assert!(valid_event_type("message_sent"));
}

#[test]
fn rejects_blank_or_long_kind() {
    assert!(!valid_event_type(" \t "));
    assert!(!valid_event_type(&"x".repeat(129)));
    assert!(valid_event_type(&"x".repeat(128)));
}

fn input(source: &str, kind: &str) -> EventInput {
    EventInput {
        schema_version: 1,
        event_id: Uuid::new_v4(),
        source_app: source.into(),
        event_type: kind.into(),
    }
}

#[test]
fn validates_event_inputs() {
    assert!(valid_event(&input("web_chat", "message_sent")));
    assert!(valid_event(&input("desktop_pet", "scan_preview")));
    assert!(!valid_event(&input("unknown", "message_sent")));
    assert!(!valid_event(&input("web_chat", "  ")));
    let mut unsupported = input("web_chat", "message.persisted");
    unsupported.schema_version = 2;
    assert!(!valid_event(&unsupported));
}

#[tokio::test]
async fn valid_event_remains_unimplemented() {
    dotenv::dotenv().ok();
    let database_url = std::env::var("ANALYTICS_DATABASE_URL").unwrap();
    let chat = std::env::var("ANALYTICS_CHAT_TOKEN").unwrap();
    let pet = std::env::var("ANALYTICS_PET_TOKEN").unwrap();
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    assert_eq!(
        ingest(
            State(IngestState {
                pool,
                tokens: Arc::new(IngestTokens { chat, pet }),
            }),
            HeaderMap::new(),
            Json(input("web_chat", "message_sent"))
        )
        .await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn invalid_event_is_rejected() {
    dotenv::dotenv().ok();
    let database_url = std::env::var("ANALYTICS_DATABASE_URL").unwrap();
    let chat = std::env::var("ANALYTICS_CHAT_TOKEN").unwrap();
    let pet = std::env::var("ANALYTICS_PET_TOKEN").unwrap();
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();

    assert_eq!(
        ingest(
            State(IngestState {
                pool,
                tokens: Arc::new(IngestTokens { chat, pet }),
            }),
            HeaderMap::new(),
            Json(input("unknown", "x"))
        )
        .await,
        StatusCode::BAD_REQUEST
    );
}
