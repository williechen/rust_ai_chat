use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use stats_server::build_app;
use tower::ServiceExt;

const ID: &str = "00000000-0000-4000-8000-000000000001";

fn post(json: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/events")
        .header("content-type", "application/json")
        .body(Body::from(json.to_owned()))
        .unwrap()
}

async fn response_status(req: Request<Body>) -> StatusCode {
    build_app().oneshot(req).await.unwrap().status()
}

#[tokio::test]
async fn health_is_200() {
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    assert_eq!(response_status(req).await, StatusCode::OK);
}

#[tokio::test]
async fn valid_event_still_returns_501() {
    let body = format!(
        r#"{{"schema_version":1,"event_id":"{ID}","source_app":"web_chat",
                "event_type":"message_sent"}}"#
    );
    assert_eq!(
        response_status(post(&body)).await,
        StatusCode::NOT_IMPLEMENTED
    );
}

#[tokio::test]
async fn invalid_source_is_400() {
    let body = format!(
        r#"{{"schema_version":1,"event_id":"{ID}","source_app":"other",
                "event_type":"message_sent"}}"#
    );
    assert_eq!(response_status(post(&body)).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_field_is_422() {
    let body = format!(
        r#"{{"schema_version":1,"event_id":"{ID}","source_app":"web_chat",
                "event_type":"x","secret":"must_not_accept"}}"#
    );
    assert_eq!(
        response_status(post(&body)).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn malformed_json_is_400() {
    assert_eq!(response_status(post("{")).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn payload_over_4k_is_413() {
    let body = format!(
        r#"{{"schema_version":1,"event_id":"{ID}","source_app":"web_chat",
                "event_type":"{}"}}"#,
        "x".repeat(5000)
    );
    assert_eq!(
        response_status(post(&body)).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn unsupported_schema_version_returns_400() {
    // 格式合法但 Schema v2 未獲支援，不能當成功處理。
    let body = format!(
        r#"{{"schema_version":2,"event_id":"{ID}","source_app":"web_chat",
            "event_type":"message.persisted"}}"#
    );
    assert_eq!(response_status(post(&body)).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_schema_version_returns_422() {
    // Schema v1 要求明確版本欄位，禁止靜默預設。
    let body = format!(
        r#"{{"event_id":"{ID}","source_app":"web_chat",
            "event_type":"message.persisted"}}"#
    );
    assert_eq!(
        response_status(post(&body)).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
