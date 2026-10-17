use stats_server::{IngestState, IngestTokens, build_app};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // 啟動時 fail-closed；不得把密鑰值寫入日誌。
    let database_url = std::env::var("ANALYTICS_DATABASE_URL")?;
    let chat = std::env::var("ANALYTICS_CHAT_TOKEN")?;
    let pet = std::env::var("ANALYTICS_PET_TOKEN")?;

    if chat.is_empty() || pet.is_empty() || chat == pet || chat.len() > 512 || pet.len() > 512 {
        return Err("來源憑證設定不合法".into());
    }

    let pool = sqlx::PgPool::connect(&database_url).await?;
    // Migration 由既有 DB 管理流程預先套用；不可修改已套用 migration。
    let app = build_app(IngestState {
        pool,
        tokens: Arc::new(IngestTokens { chat, pet }),
    });

    println!("Stats server running on http://127.0.0.1:3100");

    // 尚無 auth 和儲存，不可綁定公開地址。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3100").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
