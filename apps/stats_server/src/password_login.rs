use crate::{IngestState, pg_store};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Form, Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
const ORIGIN: &str = "http://127.0.0.1:3100";
fn digest(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn cookie(raw: &str) -> String {
    format!("analytics_session={raw}; HttpOnly; SameSite=Strict; Path=/; Max-Age=1800")
}
fn raw_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|v| v.trim().split_once('='))
        .find(|(k, _)| *k == "analytics_session")?
        .1;
    (raw.len() == 64 && raw.bytes().all(|c| c.is_ascii_hexdigit())).then(|| raw.to_owned())
}
fn renewed(response: impl IntoResponse, raw: &str) -> Response {
    let mut response = response.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie(raw).parse().expect("validated hex token"),
    );
    response
}
#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}
async fn login_page() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html lang="zh-Hant"><meta charset="utf-8"><h1>Analytics 帳密登入</h1><form method="post" action="/login"><label>帳號<input name="username" autocomplete="username" required></label><label>密碼<input type="password" name="password" autocomplete="current-password" required></label><button>登入</button></form></html>"#,
    )
}
async fn login(
    State(state): State<IngestState>,
    headers: HeaderMap,
    Form(form): Form<Login>,
) -> Result<Response, StatusCode> {
    if headers.get(header::ORIGIN).and_then(|h| h.to_str().ok()) != Some(ORIGIN) {
        return Err(StatusCode::FORBIDDEN);
    }
    if !form.username.trim().eq_ignore_ascii_case("admin")
        || !(8..=128).contains(&form.password.len())
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Existing bootstrap only provisions admin. Reject duplicate case-folded identities rather than guessing one.
    let found: Vec<(String,String,bool,String)> = sqlx::query_as("SELECT id,user_pd,is_init_pd,user_status FROM uac_user WHERE lower(user_acnt)='admin' LIMIT 2")
        .fetch_all(&state.pool).await.map_err(|_|StatusCode::SERVICE_UNAVAILABLE)?;
    if found.len() != 1 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let (id, encoded, restricted, status) = found.into_iter().next().unwrap();
    if status != "active" {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let fingerprint = digest(&encoded);
    let verified = tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&encoded).map_err(|_| ())?;
        Argon2::default()
            .verify_password(form.password.as_bytes(), &parsed)
            .map_err(|_| ())
    })
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    verified.map_err(|_| StatusCode::UNAUTHORIZED)?;
    let raw = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("DELETE FROM uac_login_session WHERE expires_at <= floor(extract(epoch from now()))::bigint").execute(&mut *tx).await.map_err(|_|StatusCode::SERVICE_UNAVAILABLE)?;
    sqlx::query("INSERT INTO uac_login_session(token_hash,user_id,credential_fingerprint,restricted,expires_at) VALUES ($1,$2,$3,$4,floor(extract(epoch from now()))::bigint+1800)")
        .bind(digest(&raw)).bind(id).bind(fingerprint).bind(restricted).execute(&mut *tx).await.map_err(|_|StatusCode::SERVICE_UNAVAILABLE)?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(renewed(
        Redirect::to(if restricted {
            "/change-password"
        } else {
            "/dashboard"
        }),
        &raw,
    ))
}
// Recheck account status, mandatory-change state and password fingerprint on each protected request.
async fn session(state: &IngestState, headers: &HeaderMap) -> Result<(bool, String), StatusCode> {
    let raw = raw_cookie(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let found:Option<(bool,bool,String,String,String)> = sqlx::query_as("SELECT s.restricted,u.is_init_pd,u.user_pd,s.credential_fingerprint,u.id FROM uac_login_session s JOIN uac_user u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>floor(extract(epoch from now()))::bigint AND u.user_status='active' AND lower(u.user_acnt)='admin'")
        .bind(digest(&raw)).fetch_optional(&state.pool).await.map_err(|_|StatusCode::SERVICE_UNAVAILABLE)?;
    let (restricted, must_change, current, fingerprint, id) =
        found.ok_or(StatusCode::UNAUTHORIZED)?;
    if digest(&current) != fingerprint {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let refreshed=sqlx::query("UPDATE uac_login_session SET expires_at=floor(extract(epoch from now()))::bigint+1800 WHERE token_hash=$1 AND expires_at>floor(extract(epoch from now()))::bigint AND EXISTS (SELECT 1 FROM uac_user WHERE id=$2 AND user_pd=$3 AND user_status='active')")
        .bind(digest(&raw)).bind(id).bind(current).execute(&state.pool).await.map_err(|_|StatusCode::SERVICE_UNAVAILABLE)?;
    if refreshed.rows_affected() != 1 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok((restricted || must_change, raw))
}
async fn change_page(
    State(state): State<IngestState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let (restricted, raw) = session(&state, &headers).await?;
    if !restricted {
        return Ok(renewed(Redirect::to("/dashboard"), &raw));
    }
    Ok(renewed(
        Html(
            "<!doctype html><html lang=\"zh-Hant\"><meta charset=\"utf-8\"><h1>首次登入須更換密碼與Email</h1><p>帳密已驗證，後台查詢仍被拒絕。本切片尚未提供改密碼提交，請勿清除資料庫標記繞過限制。</p><form method=\"post\" action=\"/logout\"><button>登出</button></form></html>",
        ),
        &raw,
    ))
}
async fn dashboard(State(state): State<IngestState>, headers: HeaderMap) -> Response {
    let (restricted, raw) = match session(&state, &headers).await {
        Ok(v) => v,
        Err(StatusCode::UNAUTHORIZED) => return Redirect::to("/login").into_response(),
        Err(e) => return e.into_response(),
    };
    if restricted {
        return renewed(Redirect::to("/change-password"), &raw);
    }
    let rows = match pg_store::summary_for_source(&state.pool, "web_chat").await {
        Ok(v) => v,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let Some(total) = crate::summary_total(&rows) else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    renewed(
        Html(format!(
            "<!doctype html><html lang=\"zh-Hant\"><meta charset=\"utf-8\"><h1>Analytics 管理後台</h1><p>已完成密碼初始化的管理員：Chat 已提交事件總數 {total}</p><p>詳細查詢畫面與管理操作將於後續切片接回。</p><form method=\"post\" action=\"/logout\"><button>登出</button></form></html>"
        )),
        &raw,
    )
}
async fn summary(
    State(state): State<IngestState>,
    Path(source): Path<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let (restricted, raw) = session(&state, &headers).await?;
    if restricted {
        return Err(StatusCode::FORBIDDEN);
    }
    if !matches!(source.as_str(), "web_chat" | "desktop_pet") {
        return Err(StatusCode::BAD_REQUEST);
    }
    let rows = pg_store::summary_for_source(&state.pool, &source)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(renewed(Json(rows), &raw))
}
async fn logout(
    State(state): State<IngestState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    if headers.get(header::ORIGIN).and_then(|h| h.to_str().ok()) != Some(ORIGIN) {
        return Err(StatusCode::FORBIDDEN);
    }
    let raw = raw_cookie(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    sqlx::query("DELETE FROM uac_login_session WHERE token_hash=$1")
        .bind(digest(&raw))
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, "/login"),
            (
                header::SET_COOKIE,
                "analytics_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
            ),
        ],
    )
        .into_response())
}

pub fn build(state: IngestState) -> Router {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout))
        .route("/change-password", get(change_page))
        .route("/", get(dashboard))
        .route("/dashboard", get(dashboard))
        .route("/api/v1/events/{source}/summary", get(summary))
        .route("/healthz", get(crate::health))
        .route("/api/v1/events", post(crate::ingest))
        .layer(DefaultBodyLimit::max(4096))
        .layer(axum::middleware::map_response(
            |mut r: Response| async move {
                r.headers_mut()
                    .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
                r
            },
        ))
        .with_state(state)
}
