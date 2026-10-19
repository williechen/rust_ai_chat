use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    routing::get,
};
use sqlx::SqlitePool;

#[derive(serde::Deserialize)]
struct Signup {
    username: String,
    email: String,
    display_name: String,
    password: String,
}

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/signup", get(page).post(signup))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(pool)
}

async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
        <html lang="zh-Hant">
            <head>
                <meta charset="utf-8">
                <title>會員註冊</title>
            </head>
            <body>
                <h1>會員註冊</h1>
                <form method="post" action="/signup">
                    <label>帳號
                        <input name="username" required minlength="3" maxlength="32" autocomplete="username">
                    </label>
                    <label>Email
                        <input name="email" type="email" required maxlength="254" autocomplete="email">
                    </label>
                    <label>顯示名稱
                        <input name="display_name" required maxlength="32">
                    </label>
                    <label>密碼
                        <input name="password" type="password" required minlength="12" maxlength="128" autocomplete="new-password">
                    </label>
                    <button type="submit">建立會員</button>
                </form>
                <a href="/signin">回登入頁</a>
            </body>
        </html>"#,
    )
}

async fn signup(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Form(form): Form<Signup>,
) -> Result<Redirect, StatusCode> {
    // 檢查請求來源，僅允許來自 http://127.0.0.1:3000 的請求
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some("http://127.0.0.1:3000") {
        return Err(StatusCode::FORBIDDEN);
    }

    let username = form.username.trim().to_owned();
    let email = form.email.trim().to_ascii_lowercase();
    let display = form.display_name.trim().to_owned();
    // 本切片 Email 為基本格式檢查；驗證信另列後續。
    if username.is_empty() || email.is_empty() || display.is_empty() || form.password.is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let password_hash = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(form.password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|hash| hash.to_string())
            .map_err(|_| ())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        r#"INSERT INTO mbr_user (id
            ,user_acnt
            ,user_pd
            ,user_email
            ,user_name
            ,user_status
            ,is_init_pd
            ,created_at
            ,updated_at
        ) VALUES (?1,?2,?3,?4,?5,'active',FALSE,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)"#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(username)
    .bind(password_hash)
    .bind(email)
    .bind(display)
    .execute(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db) if db.is_unique_violation() => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Redirect::to("/signin"))
}

#[cfg(test)]
mod day22_tests {
    use super::*;
    use argon2::{PasswordHash, PasswordVerifier};
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn day22_signup_persists_and_rejects_duplicate() {
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
            "../../../migrations/20261020060000_member_identity.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let app = routes(pool.clone());
        let body = "username=learner&email=learner%40example.test&display_name=Learner&password=lesson-only-password";
        for (input, expected) in [
            (body.to_owned(), StatusCode::SEE_OTHER),
            (body.replace("learner", "LEARNER"), StatusCode::CONFLICT),
            (
                body.replace("username=learner", "username=x"),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::post("/signup")
                        .header("origin", "http://127.0.0.1:3000")
                        .header("content-type", "application/x-www-form-urlencoded")
                        .body(Body::from(input))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::SEE_OTHER {
                assert_eq!(response.headers()["location"], "/signin");
            }
        }
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM mbr_user")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
        let (hash,): (String,) = sqlx::query_as("SELECT user_pd FROM mbr_user")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(hash.starts_with("$argon2id$"));
        Argon2::default()
            .verify_password(b"lesson-only-password", &PasswordHash::new(&hash).unwrap())
            .unwrap();
        let response = app
            .oneshot(
                Request::post("/signup")
                    .header("origin", "http://evil.invalid")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
