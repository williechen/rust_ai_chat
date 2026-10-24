#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // 啟動時 fail-closed；不得把密鑰值寫入日誌。
    let database_url = std::env::var("PET_DATABASE_URL")?;

    let db = sqlx::SqlitePool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    desktop_pet::application(db);

    Ok(())
}
