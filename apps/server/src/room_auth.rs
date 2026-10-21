use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub async fn require_member(
    State(pool): State<SqlitePool>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let raw = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookie| {
            cookie
                .split(';')
                .filter_map(|segment| segment.trim().split_once('='))
                .find(|(name, _)| *name == "chat_session")
                .map(|(_, value)| value.to_owned())
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if raw.len() != 64 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let digest = Sha256::digest(raw.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let member: Option<String> = sqlx::query_scalar(
        "SELECT u.id FROM mbr_session s JOIN mbr_user u ON u.id=s.mbr_user_id WHERE s.token_hash=?1 AND s.expires_at>unixepoch() AND u.user_status='active'"
    ).bind(digest).fetch_optional(&pool).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    member.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(next.run(request).await)
}

#[cfg(test)]
mod day25_tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use sha2::{Digest, Sha256};
    use tower::ServiceExt;

    #[tokio::test]
    async fn room_page_requires_active_sqlite_session() {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../../../migrations/20261018073931_create_auth_user.sql"
        ))
        .execute(&db)
        .await
        .unwrap();
        sqlx::raw_sql(include_str!(
            "../../../migrations/20261020152737_mbr_sessions.sql"
        ))
        .execute(&db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO mbr_user (id,user_acnt,user_pd,user_email,user_name,user_status,user_sexy,user_birthday,is_init_pd,created_at,updated_at) VALUES ('m1','member','unused','m1@example.test','Member','active','', '2000-01-01',FALSE,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
            .execute(&db).await.unwrap();
        let token = "a".repeat(64);
        let hash = Sha256::digest(token.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        sqlx::query("INSERT INTO mbr_session(id,mbr_user_id,token_hash,expires_at) VALUES('s1','m1',?1,unixepoch()+60)")
            .bind(hash).execute(&db).await.unwrap();
        let app = Router::new()
            .route("/room/{room_id}", get(|| async { StatusCode::OK }))
            .route_layer(axum::middleware::from_fn_with_state(
                db.clone(),
                require_member,
            ));
        for (cookie, expected) in [
            (None, StatusCode::UNAUTHORIZED),
            (
                Some("chat_session=invalid".to_owned()),
                StatusCode::UNAUTHORIZED,
            ),
            (Some(format!("chat_session={token}")), StatusCode::OK),
        ] {
            let mut request = Request::get("/room/real-room");
            if let Some(value) = cookie {
                request = request.header("cookie", value);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
        }
        sqlx::query("UPDATE mbr_session SET expires_at=unixepoch()-1 WHERE id='s1'")
            .execute(&db)
            .await
            .unwrap();
        let expired = app
            .oneshot(
                Request::get("/room/real-room")
                    .header("cookie", format!("chat_session={token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    }
}
