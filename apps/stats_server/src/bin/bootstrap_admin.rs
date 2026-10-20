#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let url = std::env::var("ANALYTICS_DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let password = rpassword::prompt_password("初始化 admin 密碼（不回顯）：")?;
    let confirm = rpassword::prompt_password("再次輸入：")?;
    if password != confirm {
        return Err("兩次密碼不同".into());
    }
    let id = stats_server::bootstrap::initialize(&pool, password)
        .await
        .map_err(std::io::Error::other)?;
    println!("管理員已建立：{id}；首次改密碼提交仍待接通；初始Session不開放後台");
    Ok(())
}
