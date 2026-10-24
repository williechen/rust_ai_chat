#![cfg(feature = "postgres-integration")]
use analytics_server::{IngestState, IngestTokens, bootstrap, password_login};
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use std::sync::Arc;
use tower::ServiceExt;

#[sqlx::test(migrations = "../migrations")]
async fn first_admin_login_has_persistent_restricted_session(pool: sqlx::PgPool) {
    bootstrap::initialize(&pool, "lesson-test-password".into())
        .await
        .unwrap();
    let app = password_login::build(IngestState {
        pool: pool.clone(),
        tokens: Arc::new(IngestTokens {
            chat: "test-chat".into(),
            pet: "test-pet".into(),
        }),
    });
    let login = app
        .clone()
        .oneshot(
            Request::post("/login")
                .header(header::ORIGIN, "http://127.0.0.1:3100")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("username=admin&password=lesson-test-password"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::SEE_OTHER);
    assert_eq!(login.headers()[header::LOCATION], "/change-password");
    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM uac_login_session")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    let change = app
        .clone()
        .oneshot(
            Request::get("/change-password")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(change.status(), StatusCode::OK);
    let dashboard = app
        .oneshot(
            Request::get("/dashboard")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dashboard.status(), StatusCode::SEE_OTHER);
    assert_eq!(dashboard.headers()[header::LOCATION], "/change-password");
}
