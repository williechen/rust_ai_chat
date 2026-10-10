use stats_server::build_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = build_app();
    // 尚無 auth 和儲存，不可綁定公開地址。
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3100").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
