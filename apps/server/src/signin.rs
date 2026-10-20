use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

fn token_digest(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/account", get(account))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(pool)
}

async fn login_page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
        <html lang="zh-Hant">
            <head>
                <meta charset="utf-8">
                <title>帳密登入</title>
            </head>
            <body>
                <h1>帳密登入</h1>
                <form action="/login" method="post">
                    <div style="margin-bottom: 1em;"><label>帳號</label><input name="username" autocomplete="username" required></div>
                    <div style="margin-bottom: 1em;"><label>密碼</label><input name="password" type="password" autocomplete="current-password" required></div>
                    <button type="submit">登入</button>
                </form>
                <a href="/signin">回登入頁</a>
            </body>
        </html>"#,
    )
}

async fn login(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Form(input): Form<Credentials>,
) -> Result<Response, StatusCode> {
    if headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some("http://127.0.0.1:3000") {
        return Err(StatusCode::FORBIDDEN);
    }

    let name = input.username.trim();
    if name.is_empty() || name.len() > 32 || input.password.is_empty() || input.password.len() > 128
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let found: Option<(String, String, bool)> = sqlx::query_as(
        "SELECT id,user_pd,(user_status='active') FROM mbr_user WHERE lower(user_acnt)=lower(?1)",
    )
    .bind(name)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (user_id, hash, active) = found.ok_or(StatusCode::UNAUTHORIZED)?;
    if !active {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let password = input.password;
    let verified = tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|_| ())?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map(|_| true)
            .map_err(|_| ())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if verified.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 兩個獨立 UUIDv4 組成不可預測的 session bearer；不直接入庫。
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let hashed = token_digest(&token);
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO mbr_session (id,mbr_user_id,token_hash,expires_at) VALUES (?1,?2,?3,unixepoch()+86400)"
    ).bind(id).bind(user_id).bind(hashed).execute(&pool).await
        .map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    let cookie = format!("chat_session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400");
    Ok((
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, "/account"),
            (header::SET_COOKIE, cookie.as_str()),
        ],
    )
        .into_response())
}

async fn account(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
) -> Result<Html<&'static str>, StatusCode> {
    let token = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            h.split(';')
                .filter_map(|x| x.trim().split_once('='))
                .find(|(key, _)| *key == "chat_session")
                .map(|(_, value)| value.to_owned())
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if token.len() != 64 || !token.bytes().all(|x| x.is_ascii_hexdigit()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let hashed = token_digest(&token);
    let member: Option<String> = sqlx::query_scalar(
        r#"SELECT u.id 
            FROM mbr_session s 
            JOIN mbr_user u 
            ON u.id=s.mbr_user_id 
            WHERE s.token_hash=?1 
            AND s.expires_at > unixepoch() 
            AND u.user_status='active'"#,
    )
    .bind(hashed)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    member.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Html(
        r#"<!doctype html>
        <html lang="zh-Hant">
            <head>
                <meta charset="utf-8">
                <title>已登入</title>
            </head>
            <body>
                <h1>已登入</h1>
                <a href="/">回首頁</a>
            </body>
        </html>"#,
    ))
}

#[cfg(test)]
mod day23_tests {
    use super::*;
    use argon2::PasswordHasher;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn day23_password_login_creates_persisted_session_and_guards_account() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../../../migrations/20261018073931_create_auth_user.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!(
            "../../../migrations/20261020152737_mbr_sessions.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let encoded = argon2::Argon2::default()
            .hash_password(b"lesson-only-password")
            .unwrap()
            .to_string();
        sqlx::query(
            "INSERT INTO mbr_user (id,user_acnt,user_pd,user_email,user_name,user_status,user_sexy,user_birthday,is_init_pd,created_at,updated_at) VALUES ('member-1','learner',?1,'learner@example.test','Learner','active',NULL,NULL,FALSE,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)"
        ).bind(encoded).execute(&pool).await.unwrap();

        let app = routes(pool.clone());
        let denied = app
            .clone()
            .oneshot(Request::get("/account").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

        let bad = app
            .clone()
            .oneshot(
                Request::post("/login")
                    .header("origin", "http://127.0.0.1:3000")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("username=learner&password=wrong-password"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

        let ok = app
            .clone()
            .oneshot(
                Request::post("/login")
                    .header("origin", "http://127.0.0.1:3000")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("username=LEARNER&password=lesson-only-password"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::SEE_OTHER);
        assert_eq!(ok.headers()["location"], "/account");
        let cookie = ok.headers()[axum::http::header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let allowed = app
            .oneshot(
                Request::get("/account")
                    .header(axum::http::header::COOKIE, cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(allowed.status(), StatusCode::OK);
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM mbr_session")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}
