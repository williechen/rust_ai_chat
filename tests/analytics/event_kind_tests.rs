use axum::Json;
use axum::http::StatusCode;
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
}

#[tokio::test]
async fn valid_event_remains_unimplemented() {
    assert_eq!(
        ingest(Json(input("web_chat", "message_sent"))).await,
        StatusCode::NOT_IMPLEMENTED
    );
}

#[tokio::test]
async fn invalid_event_is_rejected() {
    assert_eq!(
        ingest(Json(input("unknown", "x"))).await,
        StatusCode::BAD_REQUEST
    );
}
