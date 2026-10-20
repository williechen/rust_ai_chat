use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use server::{signin, signup};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;
use tower::ServiceExt;
async fn send(
    app: axum::Router,
    path: &str,
    body: Option<&str>,
    cookie: Option<&str>,
    origin: &str,
) -> axum::response::Response {
    let mut r = if body.is_some() {
        Request::post(path)
            .header("origin", origin)
            .header("content-type", "application/x-www-form-urlencoded")
    } else {
        Request::get(path)
    };
    if let Some(c) = cookie {
        r = r.header(header::COOKIE, c);
    }
    app.oneshot(
        r.body(body.map_or_else(Body::empty, |b| Body::from(b.to_owned())))
            .unwrap(),
    )
    .await
    .unwrap()
}
fn app(pool: SqlitePool) -> axum::Router {
    signup::routes(pool.clone()).merge(signin::routes(pool))
}
#[tokio::test]
async fn day23_full_runner_restart_and_session_rejections() {
    let path = std::env::temp_dir().join(format!("day23-{}.sqlite", uuid::Uuid::new_v4()));
    let opts = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts.clone())
        .await
        .unwrap();
    persistence::migrate(&pool).await.unwrap();
    let signup=send(app(pool.clone()),"/signup",Some("username=learner&email=learner%40example.test&display_name=Learner&password=lesson-only-password"),None,"http://127.0.0.1:3000").await;
    assert_eq!(signup.status(), StatusCode::SEE_OTHER);
    let good = "username=LEARNER&password=lesson-only-password";
    for (input, origin, status) in [
        (
            "username=learner&password=wrong",
            "http://127.0.0.1:3000",
            StatusCode::UNAUTHORIZED,
        ),
        (
            "username=missing&password=wrong",
            "http://127.0.0.1:3000",
            StatusCode::UNAUTHORIZED,
        ),
        (good, "http://evil.invalid", StatusCode::FORBIDDEN),
    ] {
        assert_eq!(
            send(app(pool.clone()), "/login", Some(input), None, origin)
                .await
                .status(),
            status
        );
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM mbr_session")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let ok = send(
        app(pool.clone()),
        "/login",
        Some(good),
        None,
        "http://127.0.0.1:3000",
    )
    .await;
    assert_eq!(ok.status(), StatusCode::SEE_OTHER);
    let set = ok.headers()[header::SET_COOKIE].to_str().unwrap();
    for attr in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=86400"] {
        assert!(set.contains(attr));
    }
    let cookie = set.split(';').next().unwrap().to_owned();
    let token = cookie.split_once('=').unwrap().1;
    let hash: String = sqlx::query_scalar("SELECT token_hash FROM mbr_session")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hash.len(), 64);
    assert_ne!(hash, token);
    let fk: Vec<(String, i64, String, i64)> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(fk.is_empty());
    pool.close().await;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap();
    persistence::migrate(&pool).await.unwrap();
    assert_eq!(
        send(app(pool.clone()), "/account", None, Some(&cookie), "")
            .await
            .status(),
        StatusCode::OK
    );
    for bad in [
        None,
        Some("chat_session=invalid"),
        Some("chat_session=0000000000000000000000000000000000000000000000000000000000000000"),
    ] {
        assert_eq!(
            send(app(pool.clone()), "/account", None, bad, "")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    sqlx::query("UPDATE mbr_user SET user_status='disabled'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        send(app(pool.clone()), "/account", None, Some(&cookie), "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("UPDATE mbr_user SET user_status='active'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE mbr_session SET expires_at=unixepoch()-1")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        send(app(pool.clone()), "/account", None, Some(&cookie), "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    pool.close().await;
    std::fs::remove_file(path).unwrap();
}
#[tokio::test]
async fn day23_old_migration_checksum_stops_upgrade() {
    let dir = std::env::temp_dir().join(format!("day23-migrations-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    for entry in std::fs::read_dir(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/old-migrations"),
    )
    .unwrap()
    {
        let e = entry.unwrap();
        std::fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate::Migrator::new(dir.as_path())
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = persistence::migrate(&pool).await.unwrap_err();
    assert!(
        err.to_string()
            .contains("previously applied but has been modified"),
        "{err}"
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, before);
    std::fs::remove_dir_all(dir).unwrap();
}
