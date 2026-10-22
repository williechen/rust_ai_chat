use axum::{
    Form, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use shared::auth::{cookie_token, token_digest};
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct LogoutForm {
    pub csrf: String,
}

pub fn routes(pool: SqlitePool) -> Router {
    Router::new()
        .route("/logout", post(logout))
        .with_state(pool)
}

pub async fn logout(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Form(form): Form<LogoutForm>,
) -> Result<Response, StatusCode> {
    if headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some("http://127.0.0.1:3000") {
        return Err(StatusCode::FORBIDDEN);
    }

    let token = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| cookie_token(h, "chat_session"))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if form.csrf != token_digest(&format!("logout:{token}")) {
        return Err(StatusCode::FORBIDDEN);
    }

    let result = sqlx::query("DELETE FROM mbr_session WHERE token_hash=?1")
        .bind(token_digest(&token))
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if result.rows_affected() != 1 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok((
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, "/"),
            (
                header::SET_COOKIE,
                "chat_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
            ),
            (header::CACHE_CONTROL, "no-store"),
        ],
    )
        .into_response())
}
